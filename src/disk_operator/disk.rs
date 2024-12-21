use serde::{Serialize, Deserialize};
use std::mem::size_of;

pub const BLOCK_SIZE: usize = 4096; // 4KB
pub const BLOCK_COUNT: usize = 1024;
pub const EOF_BYTE: u8 = 255;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FATItem {
    UnUsed, // 未使用
    Cluster(usize), // 指向下一块
    BadCluster, // 损坏块
    EOF //  文件结束
}

#[derive(Serialize, Deserialize)]
pub struct Disk {
    pub fat: Vec<FATItem>,
    pub data: Vec<u8>
}

impl Disk {
    pub fn new() -> Disk {
        Disk {
            fat: vec![FATItem::UnUsed; BLOCK_COUNT], // 创建FAT文件分配表
            data: vec![0u8; (BLOCK_COUNT - size_of::<FATItem>() * BLOCK_COUNT / BLOCK_SIZE - 1) * BLOCK_SIZE],
            // 创建数据区，单位为字节，计算方式为总块数减去FAT块数乘上块大小
        }
    }

    pub fn insert_data_in_offset(&mut self, data:  &[u8], offset: usize) {
        self.data.splice(offset..(offset+data.len()), data.iter().cloned());
    }

    pub fn insert_data_in_cluster(&mut self, data:&[u8], cluster: usize) {
        return self.insert_data_in_offset(data, cluster * BLOCK_SIZE);
    }
    // 传入数据，块号，是否插入EOF
    pub fn write_in_clusters(&mut self, data: &[u8], clusters: &[usize], insert_eof: bool) {
        for i in 0..clusters.len() - 1 {
            if i != clusters.len() - 1 {
                self.insert_data_in_cluster(&data[i * BLOCK_SIZE..(i+1) * BLOCK_SIZE], clusters[i]);
            }
            else {
                let mut buffer: Vec<u8> = Vec::with_capacity(BLOCK_SIZE); // 初始长度0，容量为BLOCK_SIZE
                buffer.extend((&data[i * BLOCK_SIZE..data.len()]).iter()); // extend从迭代器添加多个元素

                if insert_eof {
                    buffer.push(EOF_BYTE);
                }
                if buffer.len() < BLOCK_SIZE {
                    buffer.append(vec![0u8; BLOCK_SIZE - buffer.len()].as_mut()); // extend插入一个mut
                }
                self.insert_data_in_cluster(buffer.as_slice(), clusters[i]);
            }
        }
    }

    pub fn read_in_cluster(&self, cluster: usize) -> Vec<u8> {
        (&self.data[cluster * BLOCK_SIZE..(cluster + 1) * BLOCK_SIZE]).to_vec()
    }

    pub fn read_in_clusters(&self, clusters: &[usize]) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::with_capacity(BLOCK_SIZE * clusters.len());
        
        for cluster in clusters {
            let mut buffer = self.read_in_cluster(*cluster);
            data.append(&mut buffer);
        }
        // 从后向前找EOF
        for i in 1..BLOCK_SIZE {
            let index = data.len() - i;
            if data[index] == EOF_BYTE {
                data.truncate(index);
                break;
            }
        }
        data
    }
}