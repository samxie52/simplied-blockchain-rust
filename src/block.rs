// chrono： 用于处理日期时间，提供UTC时间戳功能
use chrono::{DateTime, Utc};
// serde: 用于序列化和反序列化，让结构体可以转换为JSON等格式
use serde::{Deserialize, Serialize};
// sha2: 提供SHA-256哈希算法实现
use sha2::{Digest,Sha256};
// std::fmt: 用于自定义显示格式
use std::fmt::{self, format};


/// # 区块结构体 (Block Structure)
/// 
/// 区块是区块链的基本组成单位，每个区块包含以下核心信息：
/// - 区块索引：标识区块在链中的位置
/// - 时间戳：记录区块创建时间
/// - 数据：存储在区块中的实际信息
/// - 前一区块哈希：连接到前一个区块，形成链式结构
/// - 当前哈希：当前区块的唯一标识
/// - Nonce：挖矿过程中的随机数，用于工作量证明
/// - 难度：控制挖矿的困难程度
/// 
/// ## 为什么需要这些字段？
/// - `index`: 帮助确定区块的顺序，防止重复或遗漏
/// - `timestamp`: 记录交易时间，具有法律意义
/// - `data`: 实际存储的信息，可以是交易记录、合约等
/// - `previous_hash`: 将区块连接起来，任何篡改都会被发现
/// - `hash`: 区块的"指纹"，用于快速验证完整性
/// - `nonce`: 挖矿的关键，通过调整这个值来满足难度要求
/// - `difficulty`: 控制网络的出块速度和安全性
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Block {
    /// 区块索引 - 从0开始的连续编号
    /// 创世区块的索引为0，后续区块依次递增
    pub index: u64,
    
    /// 区块创建时间戳 - 使用UTC时间
    /// 记录区块被创建的准确时间，用于排序和验证
    pub timestamp: DateTime<Utc>,
    
    /// 区块包含的数据 - 实际存储的信息
    /// 在真实的区块链中，这里通常存储交易记录、智能合约等
    pub data: String,
    
    /// 前一个区块的哈希值 - 形成链式结构的关键
    /// 通过这个字段，区块之间形成不可篡改的链条
    /// 如果有人试图修改历史区块，这个哈希值就会对不上
    pub previous_hash: String,
    
    /// 当前区块的哈希值 - 区块的唯一标识符
    /// 基于区块的所有信息计算得出，任何微小改动都会导致哈希值完全不同
    pub hash: String,
    
    /// 挖矿随机数 - 工作量证明的核心
    /// 矿工通过不断调整这个值来寻找满足难度要求的哈希值
    pub nonce: u64,
    
    /// 挖矿难度 - 控制挖矿的困难程度
    /// 数值越大，找到有效哈希值就越困难，挖矿时间越长
    pub difficulty: u32,
}


impl Block {
    /// # 创建新区块
    /// 
    /// 这个方法用于创建一个新的区块，但还没有进行挖矿
    /// 创建后需要调用 mine_block() 方法来完成工作量证明
    /// 
    /// ## 参数说明
    /// * `index` - 区块索引，通常是前一个区块的索引+1
    /// * `data` - 要存储在区块中的数据
    /// * `previous_hash` - 前一个区块的哈希值，用于建立链接
    /// * `difficulty` - 挖矿难度，决定需要多少个前导零
    /// 
    /// ## 返回值
    /// 返回一个新创建的区块实例，hash字段已经计算但可能不满足难度要求
    pub fn new(index: u64, data: String, previous_hash: String, difficulty: u32) -> Self {
        // 获取当前UTC时间作为区块创建时间
        let timestamp = Utc::now();

        // 创建区块实例，初始时nonce为0，hash为空
        let mut  block = Block {
            index,
            timestamp,
            data,
            previous_hash,
            hash: String::new(), //初始化为空，稍后计算
            nonce:0,    //从0开始，挖矿时会递增
            difficulty,
        };

        // 计算初始哈希值（但可能不满足难度要求）
        // 这个哈希值基于区块的所有信息，包括nonce=0
        block.hash = block.calculate_hash();
        block
    }

    /// # 创建创世区块（Genesis Block）
    /// 
    /// 创世区块是区块链的第一个区块，具有特殊性：
    /// - 索引固定为0
    /// - 没有前一个区块，所以previous_hash设为"0"
    /// - 包含特殊的创世信息
    /// - 通常难度较低，便于快速生成
    /// 
    /// ## 为什么需要创世区块？
    /// 区块链需要一个起点，创世区块就是这个起点
    /// 所有后续区块都直接或间接地连接到创世区块
    pub fn genesis_block() -> Self {
        Block::new(
            0,                                          // 创世区块索引固定为0
            "创世区块 - Genesis Block".to_string(),      // 创世区块的标识信息
            "0".to_string(),                           // 没有前置区块，用"0"表示
            1,                                         // 较低难度，便于快速生成     
        )
    }

    /// # 计算区块的哈希值
    /// 
    /// 使用SHA-256算法计算区块的哈希值。这个哈希值是区块的"指纹"，
    /// 基于区块的所有重要信息计算得出。如果区块的任何信息发生变化，
    /// 哈希值就会完全不同。
    /// 
    /// ## SHA-256算法特点
    /// - 输出固定为256位（64个十六进制字符）
    /// - 单向函数：很难从哈希值推算出原始数据
    /// - 雪崩效应：输入的微小变化会导致输出的巨大变化
    /// - 抗碰撞：很难找到两个不同的输入产生相同的哈希值
    /// 
    /// ## 哈希计算包含的字段
    /// 按顺序连接以下字段后进行哈希计算：
    /// 1. index - 区块索引
    /// 2. timestamp - 时间戳（转换为Unix时间戳）
    /// 3. data - 区块数据
    /// 4. previous_hash - 前一区块哈希
    /// 5. nonce - 随机数
    /// 6. difficulty - 难度值
    pub fn calculate_hash(&self) -> String {
        // 将区块的关键信息按顺序连接成一个字符串
        // timestamp.timestamp() 将DateTime转换为Unix时间戳
        let data = format!(
            "{}{}{}{}{}{}",
            self.index,                          // 区块索引
            self.timestamp.timestamp(),          // Unix时间戳
            self.data,                          // 区块数据
            self.previous_hash,                 // 前一区块哈希
            self.nonce,                         // 当前nonce值
            self.difficulty                     // 难度值
        );
        
        // 创建SHA-256哈希器
        let mut hasher = Sha256::new();
        // 将字符串转换为字节数组并输入哈希器
        hasher.update(data.as_bytes());
        // 计算哈希值并转换为十六进制字符串
        format!("{:x}", hasher.finalize())
    }

    
    /// # 挖矿 - 工作量证明算法（Proof of Work）
    /// 
    /// 这是区块链安全的核心机制。挖矿的目标是找到一个nonce值，
    /// 使得区块的哈希值满足难度要求（以指定数量的0开头）。
    /// 
    /// ## 工作量证明的原理
    /// 1. 设定难度（需要多少个前导零）
    /// 2. 从nonce=0开始尝试
    /// 3. 计算当前区块的哈希值
    /// 4. 检查哈希值是否满足难度要求
    /// 5. 如果不满足，nonce+1，重复步骤3-4
    /// 6. 直到找到满足要求的哈希值
    /// 
    /// ## 为什么这样设计？
    /// - **计算困难，验证简单**：找到正确答案需要大量计算，但验证只需一次哈希
    /// - **无法预测**：无法预知哪个nonce值会产生有效哈希，只能暴力尝试
    /// - **自动调节**：通过调整难度来控制出块时间
    /// 
    /// ## 安全性保证
    /// - 攻击者要修改历史区块，需要重新挖掘该区块及之后的所有区块
    /// - 只要诚实节点控制大部分算力，网络就是安全的
    pub fn mine_block(&mut self) {
        // 根据难度生成目标字符串，例如难度为3则target="000"
        let target = "0".repeat(self.difficulty as usize);
        
        // 记录挖矿开始时间，用于计算挖矿用时和哈希率
        let start_time = std::time::Instant::now();
        
        // 记录尝试的哈希次数，用于统计和显示进度
        let mut hash_count = 0u64;

        println!("🔨 开始挖掘区块 #{} (难度: {})...", self.index, self.difficulty);
        println!("🎯 目标：找到以 '{}' 开头的哈希值", target);
        
        // 挖矿主循环：不断尝试不同的nonce值
        loop {
            // 使用当前nonce值计算哈希
            self.hash = self.calculate_hash();
            hash_count += 1;

            // 每10000次哈希显示一次进度，避免输出过于频繁
            if hash_count % 10000 == 0 {
                // start_time.elapsed() 返回自 start_time 以来的时间,
                // as_secs_f64() 将时间转换为秒数
                let elapsed = start_time.elapsed().as_secs_f64();
                let hash_rate = hash_count as f64 / elapsed;
                // \r 让光标回到行首，实现原地更新进度
                print!("\r⛏️  已尝试 {} 次哈希, 速率: {:.0} H/s", hash_count, hash_rate);
                // 强制刷新输出缓冲区，确保进度实时显示
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }

            // 检查当前哈希值是否满足难度要求
            if self.hash.starts_with(&target) {
                // 挖矿成功！计算统计信息
                let elapsed = start_time.elapsed();
                let hash_rate = hash_count as f64 / elapsed.as_secs_f64();
                
                // 输出挖矿成功信息
                println!(); // 换行，避免与进度信息重叠
                println!("✅ 区块挖掘成功!");
                println!("🎯 哈希值: {}", self.hash);
                println!("🔢 Nonce: {}", self.nonce);
                println!("⏱️  耗时: {:.2}秒", elapsed.as_secs_f64());
                println!("🚀 哈希率: {:.0} H/s", hash_rate);
                println!("💎 总尝试次数: {}", hash_count);
                break; // 退出挖矿循环
            }
            
            // 如果当前哈希不满足要求，增加nonce值继续尝试
            // 这里可能会溢出，但在实际应用中，nonce溢出的概率极低
            self.nonce += 1;
        }
    }


    /// # 验证区块的哈希值是否正确
    /// 
    /// 检查区块存储的哈希值是否与重新计算的哈希值一致。
    /// 这用于检测区块是否被篡改。
    /// 
    /// ## 验证原理
    /// 如果区块的任何信息被修改（包括数据、时间戳、nonce等），
    /// 重新计算的哈希值就会与存储的哈希值不同。
    /// 
    /// ## 返回值
    /// - `true`: 哈希值正确，区块完整
    /// - `false`: 哈希值不匹配，区块可能被篡改
    pub fn is_valid(&self) -> bool {
        self.hash == self.calculate_hash()
    }


    /// # 验证区块是否满足工作量证明要求
    /// 
    /// 检查区块的哈希值是否满足指定的难度要求。
    /// 即使哈希值是正确计算的，也不一定满足工作量证明要求。
    /// 
    /// ## 工作量证明验证
    /// 检查哈希值是否以足够数量的0开头：
    /// - 难度为1：哈希值需要以"0"开头
    /// - 难度为3：哈希值需要以"000"开头
    /// - 难度越高，要求的前导零越多
    /// 
    /// ## 应用场景
    /// - 验证新接收的区块是否有效
    /// - 检查历史区块是否满足当时的难度要求
    /// 
    /// ## 返回值
    /// - `true`: 满足工作量证明要求
    /// - `false`: 不满足难度要求，可能是无效区块
    pub fn has_valid_proof_of_work(&self) -> bool {
        // 生成目标字符串，例如难度为3则target="000"
        // String.repeat() 生成重复的字符串
        let target = "0".repeat(self.difficulty as usize);
        // 检查当前哈希值是否以足够数量的0开头
        // String.starts_with() 检查字符串是否以指定前缀开头
        self.hash.starts_with(&target)
    }


    /// # 获取区块大小（估算值）
    /// 
    /// 计算区块在内存中占用的大概字节数。
    /// 这个值是估算的，实际序列化后的大小可能不同。
    /// 
    /// ## 计算方法
    /// - 结构体本身的固定大小
    /// - 加上所有String字段的长度
    /// 
    /// ## 用途
    /// - 网络传输时估算带宽需求
    /// - 存储空间规划
    /// - 性能分析和优化
    /// 
    /// ## 注意事项
    /// 这只是一个粗略的估算，实际大小还受到：
    /// - 内存对齐的影响
    /// - 序列化格式的影响
    /// - 压缩算法的影响
    pub fn get_size(&self) -> usize {

        //std是Rust标准库，作用是提供内存布局信息；mem是内存，size_of是内存大小
        //Self是当前类型，size_of::<Self>()返回当前类型在内存中占用的字节数
        
        // 结构体固定部分的大小
        std::mem::size_of::<Self>() + 
        // 数据字段的字符串长度
        self.data.len() + 
        // 前一区块哈希的字符串长度
        self.previous_hash.len() + 
        // 当前哈希的字符串长度（通常是64字符）
        self.hash.len()
    }

}


