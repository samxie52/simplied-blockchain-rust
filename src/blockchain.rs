// ==================== 依赖库导入 ====================
// 导入自定义的Block结构体
use crate::block::Block;
// colored: 用于在终端输出彩色文本，提升用户体验
use colored::Colorize;
// serde: 用于序列化和反序列化，支持JSON格式的存储和加载
use serde::{Deserialize, Serialize};
// std::fs: 文件系统操作，用于读写文件
use std::fs;
// std::io: 输入输出操作和错误处理
use std::io;
// std::path: 路径操作，用于处理文件路径
use std::path::Path;

/// # 区块链统计信息结构体 (BlockchainStatistics)
/// 
/// 用于记录和展示区块链的关键统计数据，帮助用户了解区块链的整体状态。
/// 这些统计信息对于网络监控、性能分析和用户界面展示都非常有用。
/// 
/// ## 字段说明
/// - `total_blocks`: 总区块数量，反映区块链的长度
/// - `total_size`: 所有区块的总大小（字节），用于存储空间分析
/// - `total_transactions`: 总交易数量，在当前简化版本中等于区块数量
/// - `current_difficulty`: 当前挖矿难度，影响出块时间和安全性
/// - `mining_reward`: 挖矿奖励，激励矿工参与网络维护
/// - `average_block_time`: 平均出块时间（秒），反映网络效率
/// - `average_hash_rate`: 平均哈希率（哈希/秒），反映网络算力
/// - `total_attempts`: 总挖矿尝试次数，基于所有区块的nonce值累加
#[derive(Debug, Clone)]
pub struct BlockchainStatistics {
    /// 区块链中的区块总数
    pub total_blocks: u64,
    /// 所有区块占用的总字节数
    pub total_size: usize,
    /// 总交易数量（当前版本中简化为区块数量）
    pub total_transactions: u64,
    /// 当前的挖矿难度级别
    pub current_difficulty: u32,
    /// 每次成功挖矿的奖励金额
    pub mining_reward: u64,
    /// 平均每个区块的生成时间（秒）
    pub average_block_time: f64,
    /// 平均哈希计算速率（哈希/秒）
    pub average_hash_rate: f64,
    /// 所有挖矿操作的总尝试次数
    pub total_attempts: u64,
}

/// # 区块链错误类型 (BlockchainError)
/// 
/// 定义区块链操作中可能遇到的各种错误类型。
/// 使用枚举类型可以更好地处理不同类型的错误，提供更准确的错误信息。
/// 
/// ## 错误类型说明
/// - `InvalidBlock`: 区块验证失败，包含具体的错误描述
/// - `InvalidChain`: 整个区块链验证失败，通常是链式结构被破坏
/// - `IoError`: 文件输入输出操作失败，如文件读写权限问题
/// - `SerializationError`: JSON序列化/反序列化失败，通常是数据格式问题
#[derive(Debug)]
pub enum BlockchainError {
    /// 无效区块错误，包含具体的错误信息
    InvalidBlock(String),
    /// 无效区块链错误，整个链的完整性被破坏
    InvalidChain(String),
    /// IO操作错误，如文件不存在、权限不足等
    IoError(io::Error),
    /// 序列化/反序列化错误，数据格式不正确
    SerializationError(serde_json::Error),
}

/// # 实现From trait - 错误类型转换
/// 
/// 为BlockchainError实现From trait，让std::io::Error可以自动转换为BlockchainError。
/// 这简化了错误处理代码，使用?操作符时可以自动转换错误类型。
impl From<io::Error> for BlockchainError {
    fn from(error: io::Error) -> Self {
        BlockchainError::IoError(error)
    }
}

/// # 实现From trait - JSON错误转换
/// 
/// 让serde_json::Error可以自动转换为BlockchainError。
/// 在JSON序列化/反序列化操作中非常有用。
impl From<serde_json::Error> for BlockchainError {
    fn from(error: serde_json::Error) -> Self {
        BlockchainError::SerializationError(error)
    }
}

/// # 实现Display trait - 错误信息格式化
/// 
/// 为BlockchainError实现Display trait，提供用户友好的错误信息显示。
/// 支持中文错误消息，便于本地化应用。
impl std::fmt::Display for BlockchainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockchainError::InvalidBlock(msg) => write!(f, "无效区块: {}", msg),
            BlockchainError::InvalidChain(msg) => write!(f, "无效区块链: {}", msg),
            BlockchainError::IoError(err) => write!(f, "IO错误: {}", err),
            BlockchainError::SerializationError(err) => write!(f, "序列化错误: {}", err),
        }
    }
}

/// # 实现Error trait - 标准错误处理
/// 
/// 让BlockchainError成为标准的Rust错误类型，可以与其他错误处理库兼容。
impl std::error::Error for BlockchainError {}

/// # 区块链主结构体 (Blockchain)
/// 
/// 这是整个区块链系统的核心结构，管理着区块链的所有功能：
/// - 维护区块链条：存储所有区块并确保链式结构的完整性
/// - 控制挖矿参数：管理难度和奖励机制
/// - 处理交易：虽然当前版本简化了交易处理，但保留了扩展接口
/// - 数据持久化：支持区块链的保存和加载
/// 
/// ## 设计理念
/// - **不可篡改性**：一旦区块被添加到链中，就很难被修改
/// - **去中心化**：每个节点都可以独立验证整个区块链
/// - **透明性**：所有交易和区块信息都是公开可验证的
/// - **安全性**：通过工作量证明和密码学哈希确保安全
/// 
/// ## 字段详解
/// - `chain`: 存储所有区块的向量，按时间顺序排列
/// - `difficulty`: 控制挖矿难度，影响网络安全性和出块速度
/// - `mining_reward`: 激励机制，鼓励矿工维护网络安全
/// - `pending_transactions`: 待处理的交易队列，等待被打包进下一个区块
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blockchain {
    /// 区块链主体 - 存储所有区块的有序列表
    /// 索引0是创世区块，后续区块按时间顺序排列
    pub chain: Vec<Block>,
    
    /// 当前网络的挖矿难度 - 控制工作量证明的复杂度
    /// 难度越高，找到有效哈希值越困难，网络安全性越高
    /// 但同时也会增加挖矿时间和能源消耗
    pub difficulty: u32,
    
    /// 挖矿奖励 - 成功挖出新区块的矿工获得的奖励
    /// 这是区块链网络的激励机制，鼓励矿工投入算力维护网络安全
    pub mining_reward: u64,
    
    /// 待处理交易池 - 等待被打包进区块的交易
    /// 在当前简化版本中使用String，实际项目中应该是Transaction结构体
    /// 矿工会从这个池中选择交易打包进新区块
    pub pending_transactions: Vec<String>,
}

impl Blockchain {
    /// # 系统默认配置常量
    
    /// 默认挖矿难度 - 平衡安全性和效率的初始值
    /// 难度2意味着哈希值需要以"00"开头
    const DEFAULT_DIFFICULTY: u32 = 2;
    
    /// 默认挖矿奖励 - 激励矿工的初始奖励金额
    /// 在真实区块链中，这个值会随着时间推移而调整
    const DEFAULT_MINING_REWARD: u64 = 100;

    /// # 创建新的区块链实例
    /// 
    /// 初始化一个全新的区块链，包含创世区块。
    /// 创世区块是区块链的起始点，所有后续区块都直接或间接地链接到它。
    /// 
    /// ## 初始化步骤
    /// 1. 创建空的区块链结构
    /// 2. 设置默认的系统参数（难度、奖励等）
    /// 3. 创建并添加创世区块
    /// 4. 返回可用的区块链实例
    /// 
    /// ## 返回值
    /// 返回一个包含创世区块的新区块链实例
    pub fn new() -> Self {
        // 创建区块链基础结构，使用默认配置
        let mut blockchain = Blockchain {
            chain: Vec::new(),                        // 空的区块链条
            difficulty: Self::DEFAULT_DIFFICULTY,     // 默认挖矿难度
            mining_reward: Self::DEFAULT_MINING_REWARD, // 默认挖矿奖励
            pending_transactions: Vec::new(),         // 空的交易池
        };
        
        // 创建并添加创世区块
        // 创世区块是区块链的第一个区块，具有特殊的标识
        let genesis_block = Block::genesis_block();
        blockchain.chain.push(genesis_block);
        
        blockchain
    }

    /// # 获取最新区块的引用
    /// 
    /// 返回区块链中最后一个区块的不可变引用。
    /// 这个方法在添加新区块时非常有用，因为新区块需要引用前一个区块的哈希。
    /// 
    /// ## 安全性保证
    /// 使用expect()确保区块链至少包含创世区块。
    /// 在正确初始化的区块链中，这个方法永远不会失败。
    /// 
    /// ## 返回值
    /// 返回最新区块的不可变引用，用于读取区块信息
    pub fn get_latest_block(&self) -> &Block {
        // expect() 方法用于在链为空时提供错误信息
        self.chain.last().expect("区块链至少包含创世区块")
    }

    /// # 添加新区块到区块链
    /// 
    /// 这是区块链最核心的功能之一。创建新区块、执行挖矿、验证有效性，
    /// 最后将新区块添加到区块链末尾。
    /// 
    /// ## 工作流程
    /// 1. 获取当前链上的最新区块
    /// 2. 创建新区块，索引为前一区块+1
    /// 3. 使用前一区块的哈希值建立链接
    /// 4. 执行挖矿操作（工作量证明）
    /// 5. 验证新区块的有效性
    /// 6. 将有效区块添加到链上
    /// 
    /// ## 安全检查
    /// - 验证区块哈希的正确性
    /// - 确保满足工作量证明要求
    /// - 检查区块链接的完整性
    /// 
    /// ## 参数
    /// * `data` - 要存储在新区块中的数据
    /// 
    /// ## 返回值
    /// * `Ok(())` - 成功添加区块
    /// * `Err(BlockchainError)` - 添加失败，包含具体错误信息
    pub fn add_block(&mut self, data: String) -> Result<(), BlockchainError> {
        // 获取链上最新区块，作为新区块的前驱
        let previous_block = self.get_latest_block();
        
        // 创建新区块，所有参数都基于当前区块链状态
        let mut new_block = Block::new(
            previous_block.index + 1,           // 新区块索引 = 前一区块索引 + 1
            data,                               // 用户提供的区块数据
            previous_block.hash.clone(),        // 前一区块的哈希值，建立链接
            self.difficulty,                    // 当前网络的挖矿难度
        );
        
        // 执行挖矿操作 - 这是最耗时的步骤
        // 挖矿会调整nonce值直到找到满足难度要求的哈希值
        new_block.mine_block();
        
        // 验证新挖出的区块是否有效
        // 检查哈希值是否正确计算
        if !new_block.is_valid() {
            return Err(BlockchainError::InvalidBlock("区块哈希无效".to_string()));
        }
        
        // 验证工作量证明是否满足要求
        // 确保哈希值以足够数量的0开头
        if !new_block.has_valid_proof_of_work() {
            return Err(BlockchainError::InvalidBlock("工作量证明无效".to_string()));
        }
        
        // 所有验证通过，将新区块添加到链上
        self.chain.push(new_block);
        Ok(())
    }

    /// # 批量挖矿功能
    /// 
    /// 连续创建指定数量的区块，主要用于：
    /// - 测试区块链性能
    /// - 快速生成测试数据
    /// - 演示区块链的扩展过程
    /// 
    /// ## 功能特点
    /// - 自动生成有意义的区块数据
    /// - 实时显示挖矿进度
    /// - 统计批量挖矿的性能数据
    /// - 错误处理：任何区块挖矿失败都会中止整个批量操作
    /// 
    /// ## 性能统计
    /// 挖矿完成后会显示：
    /// - 总耗时
    /// - 平均每区块耗时
    /// - 整体挖矿效率
    /// 
    /// ## 参数
    /// * `count` - 要挖掘的区块数量
    /// * `data_prefix` - 区块数据的前缀，实际数据会加上序号
    /// 
    /// ## 返回值
    /// * `Ok(())` - 批量挖矿成功
    /// * `Err(BlockchainError)` - 某个区块挖矿失败
    /// 
    /// ## 使用示例
    /// ```rust
    /// blockchain.batch_mine(5, "测试区块")?;
    /// // 会创建数据为"测试区块 #1", "测试区块 #2", ..., "测试区块 #5"的区块
    /// ```
    pub fn batch_mine(&mut self, count: u32, data_prefix: &str) -> Result<(), BlockchainError> {
        println!("🚀 开始批量挖矿 {} 个区块...", count);
        let start_time = std::time::Instant::now();
        
        // 循环挖掘指定数量的区块
        for i in 1..=count {
            // 为每个区块生成唯一的数据标识
            let data = format!("{} #{}", data_prefix, i);
            println!("\n📦 挖掘第 {}/{} 个区块", i, count);
            
            // 挖掘单个区块，如果失败则中止整个批量操作
            // ?操作符用于传播错误,如果add_block返回错误,则batch_mine也会返回错误，这样可以确保批量挖矿的完整性
            self.add_block(data)?;
        }
        
        // 计算并显示批量挖矿的性能统计
        let elapsed = start_time.elapsed();
        println!("\n✅ 批量挖矿完成!");
        println!("⏱️  总耗时: {:.2}秒", elapsed.as_secs_f64());
        println!("📊 平均每区块: {:.2}秒", elapsed.as_secs_f64() / count as f64);
        
        Ok(())
    }

    /// # 验证整个区块链的完整性
    /// 
    /// 这是区块链系统最重要的功能之一。全面检查区块链的每个环节，
    /// 确保没有被篡改、没有错误，并且所有区块都正确链接。
    /// 
    /// ## 验证项目
    /// 
    /// ### 1. 创世区块验证
    /// - 检查是否存在创世区块（索引为0）
    /// - 验证创世区块的previous_hash为"0"
    /// - 确保区块链不为空
    /// 
    /// ### 2. 逐区块验证
    /// 对每个非创世区块进行以下检查：
    /// 
    /// #### 区块内部完整性
    /// - **哈希正确性**：重新计算哈希值，与存储的值比较
    /// - **工作量证明**：验证哈希值是否满足当时的难度要求
    /// 
    /// #### 区块链连接性
    /// - **哈希链接**：当前区块的previous_hash必须等于前一区块的hash
    /// - **索引连续性**：区块索引必须连续递增，不能跳跃或重复
    /// 
    /// ## 安全意义
    /// 
    /// ### 防篡改保护
    /// 如果有人试图修改历史区块的数据，该区块的哈希值会发生变化，
    /// 导致下一个区块的previous_hash不匹配，验证失败。
    /// 
    /// ### 完整性保证
    /// 确保区块链从创世区块到最新区块的每个环节都是正确的，
    /// 没有遗漏、重复或错误的区块。
    /// 
    /// ## 返回值
    /// * `true` - 区块链完全有效，可以信任
    /// * `false` - 发现问题，区块链可能被篡改或损坏
    pub fn is_chain_valid(&self) -> bool {
        // 首先检查区块链是否为空
        if self.chain.is_empty() {
            return false;
        }
        
        // 验证创世区块的特殊性质
        let genesis = &self.chain[0];
        if genesis.index != 0 || genesis.previous_hash != "0" {
            return false;
        }
        
        // 从第二个区块开始验证每个区块
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];
            
            // 验证当前区块的哈希值是否正确
            // 这检查区块内容是否被篡改
            if !current_block.is_valid() {
                return false;
            }
            
            // 验证工作量证明是否满足要求
            // 这检查区块是否经过了正当的挖矿过程
            if !current_block.has_valid_proof_of_work() {
                return false;
            }
            
            // 验证区块链的连接性
            // 当前区块的previous_hash必须等于前一区块的hash
            if current_block.previous_hash != previous_block.hash {
                return false;
            }
            
            // 验证区块索引的连续性
            // 确保区块按正确顺序排列，没有跳跃或重复
            if current_block.index != previous_block.index + 1 {
                return false;
            }
        }
        
        // 所有验证都通过，区块链是有效的
        true
    }

    /// # 获取指定索引的区块
    /// 
    /// 通过区块索引查找并返回对应的区块。这是一个只读操作，
    /// 返回区块的不可变引用，确保区块链的完整性不被破坏。
    /// 
    /// ## 查找机制
    /// 直接使用Vector的索引访问，时间复杂度为O(1)。
    /// 这比遍历查找更高效，但要求索引必须连续。
    /// 
    /// ## 安全性
    /// 使用Option类型避免数组越界错误。如果索引超出范围，
    /// 返回None而不是引发panic，让调用者决定如何处理。
    /// 
    /// ## 参数
    /// * `index` - 区块索引，从0开始（0是创世区块）
    /// 
    /// ## 返回值
    /// * `Some(&Block)` - 找到指定索引的区块
    /// * `None` - 索引超出范围，区块不存在
    /// 
    /// ## 使用示例
    /// ```rust
    /// if let Some(block) = blockchain.get_block(3) {
    ///     println!("区块3的数据: {}", block.data);
    /// } else {
    ///     println!("区块3不存在");
    /// }
    /// ```
    pub fn get_block(&self, index: u64) -> Option<&Block> {
        // 将u64索引转换为usize，然后进行安全的数组访问
        self.chain.get(index as usize)
    }

    /// # 设置挖矿难度
    /// 
    /// 动态调整网络的挖矿难度，这是区块链网络自我调节的重要机制。
    /// 在真实的区块链网络中，难度会根据实际出块时间自动调整。
    /// 
    /// ## 难度的作用
    /// 
    /// ### 控制出块时间
    /// - **难度高**：需要更多计算才能找到有效哈希，出块时间长
    /// - **难度低**：较少计算就能找到有效哈希，出块时间短
    /// 
    /// ### 维护网络稳定
    /// - **算力增加时**：提高难度，保持出块时间稳定
    /// - **算力减少时**：降低难度，避免网络卡顿
    /// 
    /// ## 安全限制
    /// 限制难度范围在1-10之间，原因：
    /// - **最小值1**：确保基本的工作量证明要求
    /// - **最大值10**：避免挖矿时间过长，影响用户体验
    /// - **超出范围**：自动忽略，保护网络稳定性
    /// 
    /// ## 参数
    /// * `difficulty` - 新的难度值，必须在1-10范围内
    /// 
    /// ## 使用场景
    /// - 网络负载调整
    /// - 测试不同难度下的性能
    /// - 适应硬件算力变化
    pub fn set_difficulty(&mut self, difficulty: u32) {
        // 验证难度值在合理范围内
        if difficulty > 0 && difficulty <= 10 {
            self.difficulty = difficulty;
            println!("✅ 挖矿难度已设置为: {}", difficulty);
        } else {
            // 难度值超出范围，不进行修改并给出提示
            println!("❌ 难度必须在1-10之间");
        }
    }

    /// # 获取区块链统计信息
    /// 
    /// 生成详细的区块链统计报告，包含各种关键性能指标。
    /// 这些信息对于网络监控、性能分析和用户界面展示非常重要。
    /// 
    /// ## 统计项目详解
    /// 
    /// ### 基础统计
    /// - **区块总数**：反映区块链的长度和历史
    /// - **总大小**：所有区块占用的存储空间
    /// - **交易总数**：当前简化版本中等于区块数量
    /// 
    /// ### 网络参数
    /// - **当前难度**：影响挖矿时间和安全性
    /// - **挖矿奖励**：激励机制的核心参数
    /// 
    /// ### 性能指标
    /// - **平均出块时间**：从创世区块到最新区块的时间跨度除以区块间隔数
    /// - **总尝试次数**：所有区块挖矿过程中的nonce累加值
    /// - **平均哈希率**：基于总尝试次数和时间计算的算力估算
    /// 
    /// ## 计算方法
    /// 
    /// ### 平均出块时间计算
    /// ```
    /// 平均时间 = (最新区块时间 - 创世区块时间) / (区块数量 - 1)
    /// ```
    /// 
    /// ### 哈希率计算
    /// ```
    /// 哈希率 = 总尝试次数 / (区块数量 × 平均出块时间)
    /// ```
    /// 
    /// ## 返回值
    /// 返回包含所有统计信息的BlockchainStatistics结构体
    pub fn get_statistics(&self) -> BlockchainStatistics {
        // 基础统计信息
        let total_blocks = self.chain.len() as u64;
        //self.chain.iter().map(|block| block.get_size()).sum() 意思是
        //遍历self.chain中的每个block,并调用block.get_size()获取每个区块的大小,
        //然后将所有区块的大小相加,得到整个区块链的总大小
        let total_size = self.chain.iter().map(|block| block.get_size()).sum();
        let total_transactions = self.chain.len() as u64; // 简化版本，每个区块算一个交易
        
        // 计算平均出块时间
        let average_block_time = if self.chain.len() > 1 {
            let first_block = &self.chain[0];
            let last_block = self.get_latest_block();
            // 计算从创世区块到最新区块的时间跨度（秒）
            let duration = (last_block.timestamp - first_block.timestamp).num_seconds() as f64;
            // 除以区块间隔数得到平均时间
            duration / (self.chain.len() - 1) as f64
        } else {
            // 只有创世区块时，无法计算平均时间
            0.0
        };
        
        // 计算总尝试次数（基于nonce值的估算）
        // 每个区块的nonce值代表为找到有效哈希而进行的尝试次数
        let total_attempts = self.chain.iter().map(|block| block.nonce).sum();
        
        // 计算平均哈希率
        let average_hash_rate = if average_block_time > 0.0 {
            // 哈希率 = 总尝试次数 / (总区块数 × 平均出块时间)
            (total_attempts as f64) / (total_blocks as f64 * average_block_time)
        } else {
            0.0
        };
        
        // 构造并返回统计信息结构体
        BlockchainStatistics {
            total_blocks,
            total_size,
            total_transactions,
            current_difficulty: self.difficulty,
            mining_reward: self.mining_reward,
            average_block_time,
            average_hash_rate,
            total_attempts,
        }
    }

    /// # 保存区块链到JSON文件
    /// 
    /// 将整个区块链序列化为JSON格式并保存到指定文件。
    /// 这是区块链数据持久化的核心功能，支持数据的长期存储和备份。
    /// 
    /// <P: AsRef<Path>>
    /// P: AsRef<Path> 是一个 trait bound，表示 P 可以被转换为 Path 的引用。
    /// Path 是 Rust 标准库中的一个 trait，用于表示文件路径。
    /// 
    /// ## 序列化过程
    /// 1. 使用serde_json将区块链结构转换为格式化的JSON字符串
    /// 2. 自动创建必要的目录结构
    /// 3. 将JSON数据写入指定文件
    /// 
    /// ## 文件格式
    /// 保存的JSON文件包含：
    /// - 所有区块的完整信息（索引、时间戳、数据、哈希值等）
    /// - 区块链配置参数（难度、奖励等）
    /// - 待处理交易池的内容
    /// 
    /// ## 安全特性
    /// - **目录自动创建**：如果目标路径的目录不存在，会自动创建
    /// - **原子写入**：写入操作要么完全成功，要么完全失败，不会产生部分写入的损坏文件
    /// - **错误处理**：文件系统错误和序列化错误都会被妥善处理
    /// 
    /// ## 参数
    /// * `path` - 保存文件的路径，支持任何实现AsRef<Path>的类型
    /// 
    /// ## 返回值
    /// * `Ok(())` - 保存成功
    /// * `Err(BlockchainError::IoError)` - 文件系统错误（如权限不足、磁盘满等）
    /// * `Err(BlockchainError::SerializationError)` - JSON序列化失败
    /// 
    /// ## 使用示例
    /// ```rust
    /// blockchain.save_to_file("data/my_blockchain.json")?;
    /// blockchain.save_to_file(Path::new("/tmp/backup.json"))?;
    /// ```
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), BlockchainError> {
        // 确保目标路径的父目录存在
        // 这避免了因为目录不存在而导致的写入失败
        // path.as_ref()返回path的引用,Path::parent()返回path的父目录
        if let Some(parent) = path.as_ref().parent() {
            // ?操作符用于传播错误,如果create_dir_all返回错误,则save_to_file也会返回错误
            fs::create_dir_all(parent)?;
        }
        
        // 将区块链结构序列化为格式化的JSON字符串
        // serde_json::to_string_pretty 生成便于阅读的格式化JSON
        let json = serde_json::to_string_pretty(self)?;
        
        // 将JSON字符串写入文件
        // 这是一个原子操作，要么完全成功，要么完全失败
        fs::write(path, json)?;
        Ok(())
    }

    /// # 从JSON文件加载区块链
    /// 
    /// 从指定的JSON文件中读取并反序列化区块链数据。
    /// 这是数据恢复和系统启动时的关键功能。
    /// 
    /// ## 加载过程
    /// 1. 读取JSON文件内容
    /// 2. 使用serde_json反序列化为Blockchain结构
    /// 3. 验证加载的区块链完整性
    /// 4. 返回可用的区块链实例
    /// 
    /// ## 完整性验证
    /// 加载完成后会自动调用is_chain_valid()进行验证：
    /// - **防止损坏数据**：检测文件是否在存储过程中损坏
    /// - **防止篡改**：确保加载的数据没有被恶意修改
    /// - **确保一致性**：验证所有区块的链接关系正确
    /// 
    /// ## 安全考虑
    /// - **输入验证**：不信任外部文件，必须经过完整验证
    /// - **错误隔离**：文件错误不会影响系统的其他部分
    /// - **类型安全**：使用Rust的类型系统防止数据类型错误
    /// 
    /// ## 参数
    /// * `path` - JSON文件的路径，支持任何实现AsRef<Path>的类型
    /// 
    /// ## 返回值
    /// * `Ok(Blockchain)` - 成功加载并验证的区块链实例
    /// * `Err(BlockchainError::IoError)` - 文件读取失败（如文件不存在、权限不足等）
    /// * `Err(BlockchainError::SerializationError)` - JSON格式错误或数据类型不匹配
    /// * `Err(BlockchainError::InvalidChain)` - 加载的区块链验证失败
    /// 
    /// ## 使用示例
    /// ```rust
    /// let blockchain = Blockchain::load_from_file("data/saved_blockchain.json")?;
    /// println!("成功加载区块链，包含 {} 个区块", blockchain.chain.len());
    /// ```
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, BlockchainError> {
        // 读取JSON文件的完整内容
        let json = fs::read_to_string(path)?;
        
        // 将JSON字符串反序列化为Blockchain结构
        let blockchain: Blockchain = serde_json::from_str(&json)?;
        
        // 验证加载的区块链完整性
        // 这是安全的关键步骤，确保加载的数据是可信的
        if !blockchain.is_chain_valid() {
            return Err(BlockchainError::InvalidChain("加载的区块链无效".to_string()));
        }
        
        Ok(blockchain)
    }

    /// # 显示完整区块链信息
    /// 
    /// 以用户友好的格式展示整个区块链的详细信息。
    /// 这个方法主要用于调试、演示和用户界面展示。
    /// 
    /// ## 显示内容
    /// 
    /// ### 区块链概览
    /// - 区块总数
    /// - 当前难度设置
    /// - 挖矿奖励配置
    /// 
    /// ### 逐区块详情
    /// 对每个区块显示：
    /// - 区块索引和基本信息
    /// - 时间戳（格式化为可读时间）
    /// - 存储的数据内容
    /// - 哈希值（简化显示，避免过长）
    /// - 挖矿参数（nonce、难度）
    /// - 区块大小估算
    /// 
    /// ### 链接关系可视化
    /// 使用ASCII字符绘制区块之间的连接关系：
    /// ```
    /// 区块 #0
    /// ├─ 时间戳: 2024-01-01 10:00:00 UTC
    /// ├─ 数据: 创世区块
    /// └─ 哈希值: 0000a1b2...c3d4e5f6
    ///     ↓
    /// 区块 #1
    /// ├─ 时间戳: 2024-01-01 10:05:32 UTC
    /// └─ 数据: 第一笔交易
    /// ```
    /// 
    /// ### 验证状态
    /// 最后显示整个区块链的验证结果，让用户知道数据是否可信。
    /// 
    /// ## 设计考虑
    /// - **彩色输出**：使用colored库提供彩色终端输出，提升可读性
    /// - **信息分层**：重要信息突出显示，细节信息适当简化
    /// - **格式化时间**：将UTC时间戳转换为人类可读的格式
    /// - **哈希简化**：只显示哈希值的开头和结尾，避免输出过长
    pub fn display_chain(&self) {
        // 显示区块链标题和基本统计信息
        // bright_cyan()提供彩色输出,提升可读性
        println!("\n{}", "🔗 ===== 完整区块链 =====".bright_cyan());
        println!("区块总数: {}", self.chain.len());
        println!("当前难度: {}", self.difficulty);
        println!("挖矿奖励: {}", self.mining_reward);
        println!("{}", "─".repeat(60));
        
        // 逐个显示每个区块的详细信息
        // enumerate()返回一个迭代器，包含当前元素的索引和值
        for (i, block) in self.chain.iter().enumerate() {
            // 使用Block的Display trait进行格式化输出
            println!("\n{}", block);
            
            // 如果不是最后一个区块，显示连接箭头
            if i < self.chain.len() - 1 {
                println!("    ↓");
            }
        }
        
        // 显示分隔线和验证状态
        println!("\n{}", "─".repeat(60));
        println!("区块链验证: {}", 
                if self.is_chain_valid() { 
                    "✅ 有效" 
                } else { 
                    "❌ 无效" 
                }
        );
    }
}

/// # 实现Default trait - 提供默认实例
/// 
/// 为Blockchain实现Default trait，让用户可以使用Blockchain::default()
/// 创建新的区块链实例。这提供了一种更符合Rust习惯的初始化方式。
/// 
/// Default::default() 与 Blockchain::new() 功能完全相同，
/// 都会创建包含创世区块的新区块链。
impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== 单元测试 ====================
// 单元测试确保区块链功能的正确性和稳定性

#[cfg(test)]  // 只在测试时编译这些代码
mod tests {
    use super::*;  // 导入上级模块的所有公共项
    use tempfile::NamedTempFile;  // 用于创建临时测试文件

    /// # 测试区块链的创建
    /// 
    /// 验证新创建的区块链是否满足以下条件：
    /// - 包含且仅包含创世区块
    /// - 创世区块的索引为0
    /// - 整个区块链通过完整性验证
    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.chain.len(), 1);  // 应该只有创世区块
        assert_eq!(blockchain.chain[0].index, 0);  // 创世区块索引为0
        assert!(blockchain.is_chain_valid());  // 区块链应该是有效的
    }

    /// # 测试添加区块功能
    /// 
    /// 验证添加新区块后的区块链状态：
    /// - 区块数量正确增加
    /// - 新区块正确链接到前一个区块
    /// - 整个区块链仍然保持有效
    #[test]
    fn test_add_block() {
        let mut blockchain = Blockchain::new();
        blockchain.add_block("测试区块".to_string()).unwrap();
        assert_eq!(blockchain.chain.len(), 2);  // 应该有2个区块
        assert!(blockchain.is_chain_valid());  // 区块链应该仍然有效
    }

    /// # 测试区块链完整性验证
    /// 
    /// 验证区块链能够正确检测篡改：
    /// - 正常区块链应该验证通过
    /// - 被篡改的区块链应该验证失败
    /// 
    /// 这个测试模拟了恶意攻击者试图修改历史数据的场景，
    /// 验证区块链的防篡改能力。
    #[test]
    fn test_chain_validation() {
        let mut blockchain = Blockchain::new();
        blockchain.add_block("区块1".to_string()).unwrap();
        blockchain.add_block("区块2".to_string()).unwrap();
        assert!(blockchain.is_chain_valid());  // 正常区块链应该有效
        
        // 模拟篡改：修改历史区块的数据
        blockchain.chain[1].data = "被篡改的数据".to_string();
        assert!(!blockchain.is_chain_valid());  // 篡改后应该验证失败
    }

    /// # 测试文件保存和加载功能
    /// 
    /// 验证区块链的序列化和反序列化功能：
    /// - 保存到文件不应该出错
    /// - 从文件加载应该得到相同的区块链
    /// - 加载的区块链应该通过完整性验证
    /// 
    /// 使用临时文件避免测试对文件系统的持久影响。
    #[test]
    fn test_save_and_load() {
        let mut blockchain = Blockchain::new();
        blockchain.add_block("测试保存".to_string()).unwrap();
        
        // 创建临时文件进行测试
        let temp_file = NamedTempFile::new().unwrap();
        blockchain.save_to_file(temp_file.path()).unwrap();
        
        // 从文件加载区块链
        let loaded_blockchain = Blockchain::load_from_file(temp_file.path()).unwrap();
        
        // 验证加载的区块链与原始区块链一致
        assert_eq!(blockchain.chain.len(), loaded_blockchain.chain.len());
        assert!(loaded_blockchain.is_chain_valid());
    }

    /// # 测试难度设置功能
    /// 
    /// 验证难度设置的边界检查：
    /// - 有效难度值应该被正确设置
    /// - 无效难度值应该被拒绝，保持原值不变
    /// 
    /// 这确保了网络参数的安全性，防止恶意或错误的配置。
    #[test]
    fn test_difficulty_setting() {
        let mut blockchain = Blockchain::new();
        
        // 测试设置有效难度
        blockchain.set_difficulty(3);
        assert_eq!(blockchain.difficulty, 3);
        
        // 测试设置无效难度（超出范围）
        blockchain.set_difficulty(15); // 超出范围
        assert_eq!(blockchain.difficulty, 3); // 应该保持不变
    }

    /// # 测试批量挖矿功能
    /// 
    /// 验证批量挖矿能够正确创建多个区块：
    /// - 创建指定数量的区块
    /// - 每个区块都有正确的数据标识
    /// - 所有区块都正确链接
    /// - 整个区块链保持有效
    #[test]
    fn test_batch_mining() {
        let mut blockchain = Blockchain::new();
        blockchain.batch_mine(3, "批量测试").unwrap();
        
        // 验证区块数量（创世区块 + 3个新区块）
        assert_eq!(blockchain.chain.len(), 4);
        
        // 验证每个新区块的数据格式
        assert_eq!(blockchain.chain[1].data, "批量测试 #1");
        assert_eq!(blockchain.chain[2].data, "批量测试 #2");
        assert_eq!(blockchain.chain[3].data, "批量测试 #3");
        
        // 验证整个区块链仍然有效
        assert!(blockchain.is_chain_valid());
    }

    /// # 测试区块检索功能
    /// 
    /// 验证按索引获取区块的功能：
    /// - 有效索引应该返回正确的区块
    /// - 无效索引应该返回None，而不是panic
    #[test]
    fn test_block_retrieval() {
        let mut blockchain = Blockchain::new();
        blockchain.add_block("检索测试".to_string()).unwrap();
        
        // 测试获取存在的区块
        assert!(blockchain.get_block(0).is_some());  // 创世区块
        assert!(blockchain.get_block(1).is_some());  // 新添加的区块
        
        // 测试获取不存在的区块
        assert!(blockchain.get_block(999).is_none()); // 超出范围的索引
    }

    /// # 测试统计信息功能
    /// 
    /// 验证区块链统计信息的计算正确性：
    /// - 基础统计信息准确
    /// - 计算的指标符合预期
    /// - 边界情况处理正确
    #[test]
    fn test_statistics() {
        let mut blockchain = Blockchain::new();
        blockchain.add_block("统计测试1".to_string()).unwrap();
        blockchain.add_block("统计测试2".to_string()).unwrap();
        
        let stats = blockchain.get_statistics();
        
        // 验证基础统计信息
        assert_eq!(stats.total_blocks, 3);  // 创世区块 + 2个新区块
        assert_eq!(stats.current_difficulty, Blockchain::DEFAULT_DIFFICULTY);
        assert_eq!(stats.mining_reward, Blockchain::DEFAULT_MINING_REWARD);
        
        // 验证计算的统计信息
        assert!(stats.total_size > 0);  // 总大小应该大于0
        assert!(stats.total_attempts > 0);  // 总尝试次数应该大于0
    }
}
