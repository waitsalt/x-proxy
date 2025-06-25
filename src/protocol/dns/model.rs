use std::{fmt::Display, net::Ipv4Addr};

use anyhow::Result;
use bytes::{Buf, BufMut, Bytes, BytesMut};

// ============================================================================
// DNS 头部 (Header)
// RFC 1035 Section 4.1.1
// ============================================================================

// DNS 查询/响应头部，共 12 字节。
#[derive(Debug, Clone)]
pub struct DnsHeader {
    // 16 位标识符 (Transaction ID)，由客户端设置，服务器在响应中原样返回。
    pub id: u16,
    // (RD) 期望递归 (Recursion Desired)。如果设置，表示客户端希望域名服务器进行递归查询。
    pub recursion_desired: bool,
    // (TC) 消息被截断 (Truncated Message)。如果设置，表示响应消息因长度超过协议限制而被截断。
    pub truncated_message: bool,
    // (AA) 权威答案 (Authoritative Answer)。如果设置，表示响应的域名服务器是该域名的权威服务器。
    pub authoritative_answer: bool,
    // 4 位操作码 (Opcode)。0: 标准查询 (QUERY), 1: 反向查询 (IQUERY), 2: 服务器状态请求 (STATUS)。
    pub opcode: u8,
    // (QR) 查询/响应标志。0: 查询 (Query), 1: 响应 (Response)。
    pub response: bool,
    // 4 位响应码 (Response Code)。
    pub rescode: ResultCode,
    // (CD) 禁止检查 (Checking Disabled)。EDNS扩展中使用，表示禁止服务器进行DNSSEC验证。
    pub checking_disabled: bool,
    // (AD) 已认证数据 (Authenticated Data)。EDNS扩展中使用，表示响应中的数据已经过DNSSEC验证。
    pub authed_data: bool,
    // (Z) 保留位，必须为0。
    pub z: bool,
    // (RA) 可用递归 (Recursion Available)。在响应中设置，表示服务器支持递归查询。
    pub recursion_available: bool,
    // 问题部分的条目数。
    pub questions: u16,
    // 回答部分的资源记录数。
    pub answers: u16,
    // 权威部分的资源记录数。
    pub authoritative_entries: u16,
    // 附加部分的资源记录数。
    pub additional_entries: u16,
}

impl DnsHeader {
    // 创建一个默认的 DNS 头部实例。
    pub fn new() -> Self {
        DnsHeader {
            id: 0,
            recursion_desired: false,
            truncated_message: false,
            authoritative_answer: false,
            opcode: 0,
            response: false,
            rescode: ResultCode::NOERROR,
            checking_disabled: false,
            authed_data: false,
            z: false,
            recursion_available: false,
            questions: 0,
            answers: 0,
            authoritative_entries: 0,
            additional_entries: 0,
        }
    }

    // 从字节缓冲区中读取并解析 DNS 头部。
    pub fn read(&mut self, buffer: &mut Bytes) -> Result<()> {
        // 读取 12 字节的头部
        self.id = buffer.get_u16();

        let flags = buffer.get_u16();
        // flags 的高 8 位
        let a = (flags >> 8) as u8;
        // flags 的低 8 位
        let b = (flags & 0xFF) as u8;

        // 解析高 8 位中的各个标志位
        self.recursion_desired = (a & (1 << 0)) > 0;
        self.truncated_message = (a & (1 << 1)) > 0;
        self.authoritative_answer = (a & (1 << 2)) > 0;
        self.opcode = (a >> 3) & 0x0F; // 提取 4 位 opcode
        self.response = (a & (1 << 7)) > 0;

        // 解析低 8 位中的各个标志位
        self.rescode = ResultCode::from_num(b & 0x0F)?; // 提取 4 位 rescode
        self.checking_disabled = (b & (1 << 4)) > 0;
        self.authed_data = (b & (1 << 5)) > 0;
        self.z = (b & (1 << 6)) > 0;
        self.recursion_available = (b & (1 << 7)) > 0;

        // 读取各部分的条目数
        self.questions = buffer.get_u16();
        self.answers = buffer.get_u16();
        self.authoritative_entries = buffer.get_u16();
        self.additional_entries = buffer.get_u16();

        Ok(())
    }

    // 将 DNS 头部写入到字节缓冲区中。
    pub fn write(&self, buffer: &mut BytesMut) -> Result<()> {
        buffer.put_u16(self.id);

        // 组合高 8 位的标志位
        let mut a: u8 = 0;
        if self.recursion_desired {
            a |= 1 << 0;
        }
        if self.truncated_message {
            a |= 1 << 1;
        }
        if self.authoritative_answer {
            a |= 1 << 2;
        }
        a |= (self.opcode & 0x0F) << 3;
        if self.response {
            a |= 1 << 7;
        }

        // 组合低 8 位的标志位
        let mut b: u8 = 0;
        b |= self.rescode as u8; // 直接使用枚举值
        if self.checking_disabled {
            b |= 1 << 4;
        }
        if self.authed_data {
            b |= 1 << 5;
        }
        if self.z {
            b |= 1 << 6;
        }
        if self.recursion_available {
            b |= 1 << 7;
        }

        // 将两个 8 位组合成一个 16 位整数并写入
        buffer.put_u16(((a as u16) << 8) | (b as u16));

        // 写入各部分的条目数
        buffer.put_u16(self.questions);
        buffer.put_u16(self.answers);
        buffer.put_u16(self.authoritative_entries);
        buffer.put_u16(self.additional_entries);

        Ok(())
    }
}

// ============================================================================
// DNS 响应码 (Result Code)
// ============================================================================

// DNS 响应码枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ResultCode {
    NOERROR = 0,  // 没问题 (No Error)
    FORMERR = 1,  // 格式错误 (Format Error)
    SERVFAIL = 2, // 服务器失败 (Server Failure)
    NXDOMAIN = 3, // 域名不存在 (Non-Existent Domain)
    NOTIMP = 4,   // 未实现 (Not Implemented)
    REFUSED = 5,  // 拒绝 (Refused)
}

impl ResultCode {
    // 从数字转换为 ResultCode 枚举。
    pub fn from_num(num: u8) -> Result<Self> {
        match num {
            0 => Ok(ResultCode::NOERROR),
            1 => Ok(ResultCode::FORMERR),
            2 => Ok(ResultCode::SERVFAIL),
            3 => Ok(ResultCode::NXDOMAIN),
            4 => Ok(ResultCode::NOTIMP),
            5 => Ok(ResultCode::REFUSED),
            _ => Err(anyhow::anyhow!("DNS error: unknown result code: {}", num)),
        }
    }
}

impl Display for ResultCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ResultCode::NOERROR => "NOERROR",
            ResultCode::FORMERR => "FORMERR",
            ResultCode::SERVFAIL => "SERVFAIL",
            ResultCode::NXDOMAIN => "NXDOMAIN",
            ResultCode::NOTIMP => "NOTIMP",
            ResultCode::REFUSED => "REFUSED",
        };
        write!(f, "{}", s)
    }
}

// ============================================================================
// DNS 记录类型和类 (Record Type & Class)
// ============================================================================

// DNS 记录类型 (TYPE) 枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum RecordType {
    A = 1,     // IPv4 地址记录
    NS = 2,    // 域名服务器记录
    CNAME = 5, // 别名记录
    MX = 15,   // 邮件交换记录
    TXT = 16,  // 文本记录
    AAAA = 28, // IPv6 地址记录
}

impl RecordType {
    // 从数字转换为 RecordType 枚举。
    pub fn from_num(num: u16) -> Option<Self> {
        match num {
            1 => Some(RecordType::A),
            2 => Some(RecordType::NS),
            5 => Some(RecordType::CNAME),
            15 => Some(RecordType::MX),
            16 => Some(RecordType::TXT),
            28 => Some(RecordType::AAAA),
            _ => None,
        }
    }
}

// DNS 记录类 (CLASS) 枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum RecordClass {
    IN = 1, // Internet (最常用)
    CS = 2, // CSNET (已废弃)
    CH = 3, // CHAOS
    HS = 4, // Hesiod
}

impl RecordClass {
    // 从数字转换为 RecordClass 枚举。
    pub fn from_num(num: u16) -> Option<Self> {
        match num {
            1 => Some(RecordClass::IN),
            2 => Some(RecordClass::CS),
            3 => Some(RecordClass::CH),
            4 => Some(RecordClass::HS),
            _ => None,
        }
    }
}

// ============================================================================
// DNS 问题 (Question)
// RFC 1035 Section 4.1.2
// ============================================================================

// DNS 查询问题部分。
#[derive(Debug, Clone)]
pub struct DnsQuestion {
    // 查询的域名，例如 "www.google.com"。
    pub name: String,
    // 查询的记录类型。
    pub qtype: RecordType,
    // 查询的记录类，通常是 IN。
    pub qclass: RecordClass,
}

impl DnsQuestion {
    // 创建一个新的 DNS 问题实例。
    pub fn new(name: String, qtype: RecordType) -> Self {
        DnsQuestion {
            name,
            qtype,
            qclass: RecordClass::IN,
        }
    }

    // 从字节缓冲区中读取并解析 DNS 问题。
    // 需要 `full_packet` 参数来处理域名压缩指针。
    pub fn read(&mut self, buffer: &mut Bytes, full_packet: &Bytes) -> Result<()> {
        self.name = read_qname(buffer, full_packet)?;

        let qtype_num = buffer.get_u16();
        self.qtype = RecordType::from_num(qtype_num)
            .ok_or_else(|| anyhow::anyhow!("Dns error: Invalid qtype: {}", qtype_num))?;

        let qclass_num = buffer.get_u16();
        self.qclass = RecordClass::from_num(qclass_num)
            .ok_or_else(|| anyhow::anyhow!("Dns error: Invalid qclass: {}", qclass_num))?;

        Ok(())
    }

    // 将 DNS 问题写入到字节缓冲区中。
    pub fn write(&self, buffer: &mut BytesMut) -> Result<()> {
        // 注意：这个实现不进行域名压缩。
        write_qname(buffer, &self.name)?;

        buffer.put_u16(self.qtype as u16);
        buffer.put_u16(self.qclass as u16);

        Ok(())
    }
}

// ============================================================================
// DNS 资源记录 (Resource Record)
// RFC 1035 Section 4.1.3
// ============================================================================

// DNS 资源记录 (RR)，用于回答、权威和附加部分。
#[derive(Debug, Clone)]
pub struct DnsRecord {
    // 记录对应的域名。
    pub name: String,
    // 记录类型。
    pub rtype: RecordType,
    // 记录类。
    pub class: RecordClass,
    // 生存时间 (Time to Live)，以秒为单位，表示记录可以被缓存多久。
    pub ttl: u32,
    // 记录的具体数据。
    pub data: DnsRecordData,
}

// DNS 资源记录的数据部分，根据记录类型而不同。
#[derive(Debug, Clone)]
pub enum DnsRecordData {
    A(Ipv4Addr),
    NS(String),
    CNAME(String),
    MX { preference: u16, exchange: String },
    TXT(String),
    AAAA([u8; 16]),   // IPv6 地址
    Unknown(Vec<u8>), // 未知类型的记录数据
}

impl DnsRecord {
    // 从字节缓冲区中读取并解析 DNS 资源记录。
    // 需要 `full_packet` 参数来处理域名压缩指针。
    pub fn read(buffer: &mut Bytes, full_packet: &Bytes) -> Result<Self> {
        let name = read_qname(buffer, full_packet)?;

        let rtype_num = buffer.get_u16();
        let class_num = buffer.get_u16();
        let ttl = buffer.get_u32();
        let data_len = buffer.get_u16() as usize;

        // 在解析记录数据前，先将数据部分切片出来，防止越界读取。
        let mut data_buffer = buffer.copy_to_bytes(data_len);

        let rtype = RecordType::from_num(rtype_num)
            .ok_or_else(|| anyhow::anyhow!("Dns error: Invalid record type: {}", rtype_num))?;
        let class = RecordClass::from_num(class_num)
            .ok_or_else(|| anyhow::anyhow!("Dns error: Invalid record class: {}", class_num))?;

        let data = match rtype {
            RecordType::A => {
                // IPv4 地址是 4 字节，可以直接从 u32 转换。
                DnsRecordData::A(Ipv4Addr::from(data_buffer.get_u32()))
            }
            RecordType::NS => {
                let ns_name = read_qname(&mut data_buffer, full_packet)?;
                DnsRecordData::NS(ns_name)
            }
            RecordType::CNAME => {
                let cname = read_qname(&mut data_buffer, full_packet)?;
                DnsRecordData::CNAME(cname)
            }
            RecordType::MX => {
                let preference = data_buffer.get_u16();
                let exchange = read_qname(&mut data_buffer, full_packet)?;
                DnsRecordData::MX {
                    preference,
                    exchange,
                }
            }
            RecordType::TXT => {
                // TXT 记录由一个或多个长度前缀的字符串组成。
                let mut txt_parts = Vec::new();
                while data_buffer.has_remaining() {
                    let len = data_buffer.get_u8() as usize;
                    if data_buffer.len() < len {
                        return Err(anyhow::anyhow!("Invalid TXT record chunk length"));
                    }
                    let part = data_buffer.copy_to_bytes(len);
                    // 这里我们假设 TXT 记录是 UTF-8，如果不是则会有损失。
                    txt_parts.push(String::from_utf8_lossy(&part).to_string());
                }
                DnsRecordData::TXT(txt_parts.join(""))
            }
            RecordType::AAAA => {
                // IPv6 地址是 16 字节。
                if data_buffer.len() != 16 {
                    return Err(anyhow::anyhow!("Invalid AAAA record length"));
                }
                let mut addr = [0u8; 16];
                data_buffer.copy_to_slice(&mut addr);
                DnsRecordData::AAAA(addr)
            }
        };

        Ok(DnsRecord {
            name,
            rtype,
            class,
            ttl,
            data,
        })
    }

    // 将 DNS 资源记录写入到字节缓冲区中。
    pub fn write(&self, buffer: &mut BytesMut) -> Result<()> {
        write_qname(buffer, &self.name)?;

        buffer.put_u16(self.rtype as u16);
        buffer.put_u16(self.class as u16);
        buffer.put_u32(self.ttl);

        // 先写入一个 16 位的长度占位符，后面再回来修改
        let pos = buffer.len();
        buffer.put_u16(0);

        match &self.data {
            DnsRecordData::A(addr) => {
                buffer.put_slice(&addr.octets());
            }
            DnsRecordData::NS(ns_name) => {
                write_qname(buffer, ns_name)?;
            }
            DnsRecordData::CNAME(cname) => {
                write_qname(buffer, cname)?;
            }
            DnsRecordData::MX {
                preference,
                exchange,
            } => {
                buffer.put_u16(*preference);
                write_qname(buffer, exchange)?;
            }
            DnsRecordData::TXT(text) => {
                // TXT 记录的字符串需要被切分成最多 255 字节的块
                for chunk in text.as_bytes().chunks(255) {
                    buffer.put_u8(chunk.len() as u8); // 写入块长度
                    buffer.put_slice(chunk); // 写入块内容
                }
            }
            DnsRecordData::AAAA(addr) => {
                buffer.put_slice(addr);
            }
            DnsRecordData::Unknown(data) => {
                buffer.put_slice(data);
            }
        }

        // 计算数据部分的实际长度
        let data_len = buffer.len() - pos - 2;
        // 回到之前的位置，写入正确的长度
        let mut len_bytes = (data_len as u16).to_be_bytes();
        buffer[pos..pos + 2].copy_from_slice(&len_bytes);

        Ok(())
    }
}

// ============================================================================
// DNS 数据包 (Packet)
// ============================================================================

// 代表一个完整的 DNS 数据包。
#[derive(Debug, Clone)]
pub struct DnsPacket {
    pub header: DnsHeader,
    pub questions: Vec<DnsQuestion>,
    pub answers: Vec<DnsRecord>,
    pub authorities: Vec<DnsRecord>,
    pub resources: Vec<DnsRecord>,
}

impl DnsPacket {
    // 创建一个空的 DNS 数据包。
    pub fn new() -> Self {
        DnsPacket {
            header: DnsHeader::new(),
            questions: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            resources: Vec::new(),
        }
    }

    // 从字节缓冲区中解析出完整的 DNS 数据包。
    pub fn from_buffer(buffer: &mut Bytes) -> Result<Self> {
        // 复制一份完整的缓冲区，用于后续处理域名压缩指针
        let full_packet = buffer.clone();

        let mut result = DnsPacket::new();
        result.header.read(buffer)?;

        for _ in 0..result.header.questions {
            let mut question = DnsQuestion::new(String::new(), RecordType::A);
            // 问题的域名也可能被压缩，所以需要传递 full_packet
            question.read(buffer, &full_packet)?;
            result.questions.push(question);
        }

        for _ in 0..result.header.answers {
            let record = DnsRecord::read(buffer, &full_packet)?;
            result.answers.push(record);
        }

        for _ in 0..result.header.authoritative_entries {
            let record = DnsRecord::read(buffer, &full_packet)?;
            result.authorities.push(record);
        }

        for _ in 0..result.header.additional_entries {
            let record = DnsRecord::read(buffer, &full_packet)?;
            result.resources.push(record);
        }

        Ok(result)
    }

    // 将 DNS 数据包序列化为字节缓冲区。
    pub fn to_buffer(&self) -> Result<BytesMut> {
        // 通常 DNS over UDP 的包大小限制在 512 字节
        let mut buffer = BytesMut::with_capacity(512);

        self.header.write(&mut buffer)?;

        for question in &self.questions {
            question.write(&mut buffer)?;
        }
        for answer in &self.answers {
            answer.write(&mut buffer)?;
        }
        for authority in &self.authorities {
            authority.write(&mut buffer)?;
        }
        for resource in &self.resources {
            resource.write(&mut buffer)?;
        }

        Ok(buffer)
    }

    // 从回答部分获取第一个 A 记录的 IPv4 地址。
    pub fn get_first_a(&self) -> Option<Ipv4Addr> {
        self.answers.iter().find_map(|record| {
            if let DnsRecordData::A(addr) = record.data {
                Some(addr)
            } else {
                None
            }
        })
    }

    // 从权威部分获取指定域名的 NS 记录（域名服务器的主机名）。
    pub fn get_ns<'a>(&'a self, qname: &str) -> impl Iterator<Item = &'a str> {
        self.authorities.iter().filter_map(move |record| {
            // 筛选出类型为 NS 且域名匹配的记录
            if record.rtype == RecordType::NS && record.name == qname {
                if let DnsRecordData::NS(ns) = &record.data {
                    Some(ns.as_str())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    // 获取解析后的 NS 服务器 IP 地址。
    // 这通常在需要向 NS 服务器进一步查询时使用。
    // 它首先找到 `qname` 的 NS 服务器主机名，然后在附加部分查找该主机名对应的 A 记录。
    pub fn get_resolved_ns(&self, qname: &str) -> Option<Ipv4Addr> {
        self.get_ns(qname)
            .flat_map(|ns_hostname| {
                self.resources.iter().filter_map(move |record| {
                    if record.rtype == RecordType::A && record.name == ns_hostname {
                        if let DnsRecordData::A(addr) = record.data {
                            Some(addr)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
            .next()
    }
}

// ============================================================================
// 工具函数 (Utility Functions)
// ============================================================================

// 读取并解析 QNAME (DNS 域名格式)。
//
// QNAME 格式由一系列的标签组成，每个标签前有一个长度字节。以一个长度为 0 的字节结尾。
// 例如, "www.google.com" 会被编码为:
// 3, 'w', 'w', 'w', 6, 'g', 'o', 'o', 'g', 'l', 'e', 3, 'c', 'o', 'm', 0
//
// 为了节省空间，DNS 协议使用指针压缩。指针是一个 2 字节的值，
// 前两位是 `11`，后面 14 位是该域名在整个数据包中的偏移量。
//
// # Arguments
// * `buffer` - 主数据缓冲区，此函数会根据读取的域名长度（包括指针）来推进它。
// * `full_packet` - 完整的 DNS 数据包，用于在遇到指针时进行跳转。
fn read_qname(buffer: &mut Bytes, full_packet: &Bytes) -> Result<String> {
    let mut name_parts = Vec::new();
    // 使用一个本地游标进行读取，这样可以随意跳转而不影响主缓冲区的状态
    let mut local_cursor = buffer.clone();
    // 标记是否已经进行过跳转。一旦跳转，主缓冲区的消耗就固定为指针的长度(2字节)
    let mut jumped = false;
    // 主缓冲区需要前进的字节数
    let mut main_buffer_consumed = 0;
    // 防止因错误的指针导致无限循环
    let max_jumps = 10;
    let mut jumps_performed = 0;

    loop {
        if !local_cursor.has_remaining() {
            return Err(anyhow::anyhow!(
                "DNS error: Unexpected end of buffer while reading qname"
            ));
        }
        let len = local_cursor.get_u8();

        // 检查前两位是否是 `11`，如果是，则这是一个指针
        if (len & 0xC0) == 0xC0 {
            if jumps_performed >= max_jumps {
                return Err(anyhow::anyhow!(
                    "DNS error: Limit of DNS name jumps exceeded"
                ));
            }
            if !jumped {
                // 如果是第一次跳转，主缓冲区只需要前进 2 个字节（指针的长度）
                buffer.advance(2);
                jumped = true;
            }

            // 读取指针的第二个字节
            if !local_cursor.has_remaining() {
                return Err(anyhow::anyhow!("DNS error: Incomplete pointer in qname"));
            }
            let next_byte = local_cursor.get_u8();
            // 计算偏移量（指针的高 6 位 + 低 8 位）
            let offset = (((len & 0x3F) as u16) << 8) | (next_byte as u16);

            // 从完整数据包的起始位置创建新的跳转游标
            local_cursor = full_packet.slice(offset as usize..);
            jumps_performed += 1;
            continue; // 继续从新位置读取
        }

        // 如果 len 是 0，表示域名结束
        if len == 0 {
            if !jumped {
                // 如果从未跳转，主缓冲区前进到当前位置 + 1 (为了最后的 0 字节)
                buffer.advance(main_buffer_consumed + 1);
            }
            break;
        }

        // 读取标签
        let label_len = len as usize;
        if !jumped {
            // 如果未跳转，记录消耗的字节数（长度字节 + 标签本身）
            main_buffer_consumed += label_len + 1;
        }

        if local_cursor.len() < label_len {
            return Err(anyhow::anyhow!(
                "DNS error: Label length exceeds buffer size"
            ));
        }
        let label_bytes = local_cursor.copy_to_bytes(label_len);
        name_parts.push(String::from_utf8_lossy(&label_bytes).to_string());
    }

    Ok(name_parts.join("."))
}

// 将字符串形式的域名写入为 QNAME 格式。
// 注意：这个简单的实现不进行指针压缩。
fn write_qname(buffer: &mut BytesMut, qname: &str) -> Result<()> {
    for part in qname.split('.') {
        let len = part.len();
        if len > 0x3F {
            // 单个标签长度不能超过 63
            return Err(anyhow::anyhow!("DNS error: DNS label too long: {}", part));
        }

        buffer.put_u8(len as u8);
        buffer.put_slice(part.as_bytes());
    }

    buffer.put_u8(0); // 以长度为 0 的字节作为结束标记
    Ok(())
}
