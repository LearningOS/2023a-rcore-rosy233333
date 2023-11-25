use core::mem;

use super::{
    block_cache_sync_all, get_block_cache, BlockDevice, DirEntry, DiskInode, DiskInodeType,
    EasyFileSystem, DIRENT_SZ,
};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use log::trace;
use log::error;
use spin::{Mutex, MutexGuard};
use bitflags::bitflags; // 我添加的代码
/// Virtual filesystem layer over easy-fs
pub struct Inode {
    block_id: usize,
    block_offset: usize,
    fs: Arc<Mutex<EasyFileSystem>>,
    block_device: Arc<dyn BlockDevice>,
}

impl Inode {
    /// Create a vfs inode
    pub fn new(
        block_id: u32,
        block_offset: usize,
        fs: Arc<Mutex<EasyFileSystem>>,
        block_device: Arc<dyn BlockDevice>,
    ) -> Self {
        Self {
            block_id: block_id as usize,
            block_offset,
            fs,
            block_device,
        }
    }
    /// Call a function over a disk inode to read it
    fn read_disk_inode<V>(&self, f: impl FnOnce(&DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .read(self.block_offset, f)
    }
    /// Call a function over a disk inode to modify it
    fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .modify(self.block_offset, f)
    }
    /// Find inode under a disk inode by name
    fn find_inode_id(&self, name: &str, disk_inode: &DiskInode) -> Option<u32> {
        // assert it is a directory
        assert!(disk_inode.is_dir(), "{:?}", disk_inode);
        let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        let mut dirent = DirEntry::empty();
        for i in 0..file_count {
            assert_eq!(
                disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
                DIRENT_SZ,
            );
            if dirent.name() == name {
                return Some(dirent.inode_id() as u32);
            }
        }
        None
    }
    // 我添加的代码-开始
    /// 找到并删除目录项。
    fn find_and_unlink_inode_id(&self, name: &str, disk_inode: &mut DiskInode) -> Option<u32> {
        // assert it is a directory
        assert!(disk_inode.is_dir());
        let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        let mut dirent = DirEntry::empty();
        let dirent_empty = DirEntry::empty();
        for i in 0..file_count {
            assert_eq!(
                disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
                DIRENT_SZ,
            );
            if dirent.name() == name {
                disk_inode.write_at(DIRENT_SZ * i, dirent_empty.as_bytes(), &self.block_device);
                return Some(dirent.inode_id() as u32);
            }
        }
        None
    }
    // 我添加的代码-结束
    /// Find inode under current inode by name
    pub fn find(&self, name: &str) -> Option<Arc<Inode>> {
        let fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            self.find_inode_id(name, disk_inode).map(|inode_id| {
                let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
                Arc::new(Self::new(
                    block_id,
                    block_offset,
                    self.fs.clone(),
                    self.block_device.clone(),
                ))
            })
        })
    }
    /// Increase the size of a disk inode
    fn increase_size(
        &self,
        new_size: u32,
        disk_inode: &mut DiskInode,
        fs: &mut MutexGuard<EasyFileSystem>,
    ) {
        if new_size < disk_inode.size {
            return;
        }
        let blocks_needed = disk_inode.blocks_num_needed(new_size);
        let mut v: Vec<u32> = Vec::new();
        for _ in 0..blocks_needed {
            v.push(fs.alloc_data());
        }
        disk_inode.increase_size(new_size, v, &self.block_device);
    }

    /// Create inode under current inode by name
    /// 我的修改：目录项的插入位置
    pub fn create(&self, name: &str) -> Option<Arc<Inode>> {
        let mut fs = self.fs.lock();
        let op = |root_inode: &DiskInode| {
            // assert it is a directory
            assert!(root_inode.is_dir());
            // has the file been created?
            self.find_inode_id(name, root_inode)
        };
        if self.read_disk_inode(op).is_some() {
            return None;
        }
        // create a new file
        // alloc a inode with an indirect block
        let new_inode_id = fs.alloc_inode();
        // initialize inode
        let (new_inode_block_id, new_inode_block_offset) = fs.get_disk_inode_pos(new_inode_id);

        get_block_cache(new_inode_block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .modify(new_inode_block_offset, |new_inode: &mut DiskInode| {
                new_inode.initialize(DiskInodeType::File);
            });
        self.modify_disk_inode(|root_inode| {
            let file_count = (root_inode.size as usize) / DIRENT_SZ;
            // 我添加的代码-开始
            // 先看文件中有没有空位
            let mut write_pos: usize = file_count * DIRENT_SZ;
            let mut dirent_read = DirEntry::empty();
            for i in 0 .. file_count {
                root_inode.read_at(i * DIRENT_SZ, dirent_read.as_bytes_mut(), &self.block_device);
                if dirent_read.inode_id() == 0 {
                    write_pos = i * DIRENT_SZ;
                }
            }

            if write_pos == file_count * DIRENT_SZ {
                // 没有空位，需要扩展文件长度后追加在结尾
                // append file in the dirent
                let new_size = (file_count + 1) * DIRENT_SZ;
                // increase size
                self.increase_size(new_size as u32, root_inode, &mut fs);
                // write dirent
                let dirent = DirEntry::new(name, new_inode_id);
                root_inode.write_at(
                    write_pos,
                    dirent.as_bytes(),
                    &self.block_device,
                );
            }
            else {
                // 有空位，在空位添加新的文件
                let dirent = DirEntry::new(name, new_inode_id);
                root_inode.write_at(
                    write_pos,
                    dirent.as_bytes(),
                    &self.block_device,
                );
            }

            // 我添加的代码-结束

        });

        let (block_id, block_offset) = fs.get_disk_inode_pos(new_inode_id);
        block_cache_sync_all();
        // return inode
        Some(Arc::new(Self::new(
            block_id,
            block_offset,
            self.fs.clone(),
            self.block_device.clone(),
        )))
        // release efs lock automatically by compiler
    }

    /// Create link under current inode by name and source inode id
    pub fn link(&self, name: &str, source_inode_id: u32) -> Option<Arc<Inode>> {
        let mut fs = self.fs.lock();
        let op = |root_inode: &DiskInode| {
            // assert it is a directory
            assert!(root_inode.is_dir());
            // has the file been created?
            self.find_inode_id(name, root_inode)
        };
        if self.read_disk_inode(op).is_some() {
            return None;
        }

        // create a new link
        let (source_inode_block_id, source_inode_block_offset) = fs.get_disk_inode_pos(source_inode_id);

        get_block_cache(source_inode_block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .modify(source_inode_block_offset, |source_inode: &mut DiskInode| {
                source_inode.nlink += 1;
            });
        self.modify_disk_inode(|root_inode| {
            let file_count = (root_inode.size as usize) / DIRENT_SZ;
            // 我添加的代码-开始
            // 先看文件中有没有空位
            let mut write_pos: usize = file_count * DIRENT_SZ;
            let mut dirent_read = DirEntry::empty();
            for i in 0 .. file_count {
                root_inode.read_at(i * DIRENT_SZ, dirent_read.as_bytes_mut(), &self.block_device);
                if dirent_read.inode_id() == 0 {
                    write_pos = i * DIRENT_SZ;
                }
            }

            if write_pos == file_count * DIRENT_SZ {
                // 没有空位，需要扩展文件长度后追加在结尾
                // append file in the dirent
                let new_size = (file_count + 1) * DIRENT_SZ;
                // increase size
                self.increase_size(new_size as u32, root_inode, &mut fs);
                // write dirent
                let dirent = DirEntry::new(name, source_inode_id);
                root_inode.write_at(
                    write_pos,
                    dirent.as_bytes(),
                    &self.block_device,
                );
            }
            else {
                // 有空位，在空位添加新的文件
                let dirent = DirEntry::new(name, source_inode_id);
                root_inode.write_at(
                    write_pos,
                    dirent.as_bytes(),
                    &self.block_device,
                );
            }

            // 我添加的代码-结束

        });

        let (block_id, block_offset) = fs.get_disk_inode_pos(source_inode_id);
        block_cache_sync_all();
        // return inode
        Some(Arc::new(Self::new(
            block_id,
            block_offset,
            self.fs.clone(),
            self.block_device.clone(),
        )))
        // release efs lock automatically by compiler
    }

    /// List inodes under current inode
    /// 我的修改：目录项的有效性判定
    pub fn ls(&self) -> Vec<String> {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            let file_count = (disk_inode.size as usize) / DIRENT_SZ;
            let mut v: Vec<String> = Vec::new();
            for i in 0..file_count {
                let mut dirent: DirEntry = DirEntry::empty();
                assert_eq!(
                    disk_inode.read_at(i * DIRENT_SZ, dirent.as_bytes_mut(), &self.block_device,),
                    DIRENT_SZ,
                );
                // 我添加的代码-开始
                // 因为文件删除的原因，可能有被清空的目录项
                if dirent.inode_id() != 0 {
                    v.push(String::from(dirent.name()));
                }
                // 我添加的代码-结束
            }
            v
        })
    }
    /// Read data from current inode
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| disk_inode.read_at(offset, buf, &self.block_device))
    }
    /// Write data to current inode
    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let mut fs = self.fs.lock();
        let size = self.modify_disk_inode(|disk_inode| {
            self.increase_size((offset + buf.len()) as u32, disk_inode, &mut fs);
            disk_inode.write_at(offset, buf, &self.block_device)
        });
        block_cache_sync_all();
        size
    }
    /// Clear the data in current inode
    pub fn clear(&self) {
        trace!("clear 1");
        let mut fs = self.fs.lock();
        trace!("clear 2");
        self.modify_disk_inode(|disk_inode| {
            trace!("clear 3");
            let size = disk_inode.size;
            let data_blocks_dealloc = disk_inode.clear_size(&self.block_device);
            trace!("clear 4");
            assert!(data_blocks_dealloc.len() == DiskInode::total_blocks(size) as usize);
            for data_block in data_blocks_dealloc.into_iter() {
                fs.dealloc_data(data_block);
            }
            trace!("clear 5");
        });
        trace!("clear 6");
        block_cache_sync_all();
        trace!("clear end");
    }

    // 我添加的代码-开始

    /// 删除文件的引用计数块，只在删除文件时使用
    // fn release_nlink(&self) {
    //     let mut fs = self.fs.lock();
    //     self.modify_disk_inode(|disk_inode| {
    //         // 我添加的代码-开始
    //         fs.dealloc_data(disk_inode.nlink_ptr);
    //         disk_inode.nlink_ptr = 0;
    //         // 我添加的代码-结束
    //     });
    //     block_cache_sync_all();
    // }

    /// 在该目录下找到一个inode并删除
    pub fn find_and_unlink(&self, name: &str) -> isize {
        trace!("fau 1");
        //修改或删除Inode
        let inode_option = self.find(name); // find内部lock了fs
        trace!("fau 2");
        let mut delete_inode = false;
        if let Some(inode) = inode_option {
            trace!("fau 3");
            let nlink = inode.modify_disk_inode(|disk_inode| {
                disk_inode.nlink -= 1;
                disk_inode.nlink
            });
            trace!("fau 4");
            if nlink == 0 {
                delete_inode = true;
                inode.clear(); // clear内部lock了fs
                trace!("fau 5");
            }
        }
        else {
            trace!("fau end 1");
            return -1;
        }

        // 删除目录项
        let find_res = self.modify_disk_inode(|dir_disk_inode| {
            trace!("fau 6");
            self.find_and_unlink_inode_id(name, dir_disk_inode) // 这里删除了目录项
        });

        trace!("fau 7");
        if let Some(inode_id) = find_res {
            trace!("fau 8");
            if delete_inode {
                trace!("fau 9");
                let mut fs = self.fs.lock();
                fs.dealloc_inode(inode_id);
            }

        }
        else {
            error!("unreachable in find_and_unlink!");
        }

        trace!("fau end 2");
        0

    }

    /// 创建一个new_path到old_path的链接
    pub fn create_and_link(&self, old_path: &str, new_path: &str ) -> isize {
        // // trace!("cal 1");
        // if let Some(old_inode) = self.find(old_path) {
        //     old_inode.find_inode_id(name, disk_inode)
        //     // trace!("cal 2");
        //     // if let Some(new_inode) = self.create(new_path) {
        //     //     // trace!("cal 3");
        //     //     // new_inode.modify_disk_inode(|new_disk_inode| {
        //     //     //     trace!("cal 4");
        //     //     //     old_inode.read_disk_inode(|old_disk_inode| {
        //     //     //         trace!("cal 5");
        //     //     //         new_disk_inode.link_from(old_disk_inode, &self.block_device);
        //     //     //         trace!("cal 6");
        //     //     //     });
        //     //     // });

        //     //     let old_disk_inode = old_inode.read_disk_inode(|old_disk_inode| {old_disk_inode.clone()});
        //     //     new_inode.modify_disk_inode(|new_disk_inode| {
        //     //         // trace!("cal 4");
        //     //         new_disk_inode.link_from(&old_disk_inode, &self.block_device);
        //     //         // trace!("cal 5");
        //     //     });

        //     //     // trace!("cal end 1");
        //     //     0
        //     // }
        //     // else {
        //     //     // trace!("cal end 2");
        //     //     -1
        //     // }
        // }
        // else {
        //     // trace!("cal end 3");
        //     -1
        // }
        let old_id_option = self.read_disk_inode(|disk_inode| {
            self.find_inode_id(old_path, disk_inode)
        });
        if let Some(old_id) = old_id_option {
            match self.link(new_path, old_id) {
                Some(_) => 0,
                None => -1
            }
            
        }
        else {
            -1
        }

    }

    /// 获取文件的Stat结构
    pub fn get_stat(&self, st: &mut Stat) -> i32{
        st.dev = 0;
        st.ino = (self.fs.lock().get_disk_inode_id((self.block_id) as u32, self.block_offset)) as u64;
        self.read_disk_inode(|disk_inode| {
            if disk_inode.is_dir() {
                st.mode = StatMode::DIR;
            }
            else {
                st.mode = StatMode::FILE;
            }
            st.nlink = disk_inode.get_nlink(&self.block_device);
        });
        0
    } 
    // 我添加的代码-结束
}

#[repr(C)]
#[derive(Debug)]
/// 系统调用返回的结构体
pub struct Stat {
    /// 文件所在磁盘驱动器号，该实验中写死为 0 即可
    pub dev: u64,
    /// inode 文件所在 inode 编号
    pub ino: u64,
    /// 文件类型
    pub mode: StatMode,
    /// 硬链接数量，初始为1
    pub nlink: u32,
    /// 无需考虑，为了兼容性设计
    pad: [u64; 7],
}

// 我添加的代码-开始
impl Stat {
    /// 空白构造函数
    pub fn new() -> Self {
        Self{
            dev: 0,
            ino: 0,
            mode: StatMode::NULL,
            nlink: 0,
            pad: [0; 7]
        }
    }
}
// 我添加的代码-结束


bitflags! {
    /// StatMode 定义
    #[derive(Debug)]
    pub struct StatMode: u32 {
        /// null
        const NULL  = 0;
        /// directory
        const DIR   = 0o040000;
        /// ordinary regular file
        const FILE  = 0o100000;
    }
}
