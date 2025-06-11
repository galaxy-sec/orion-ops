pub mod depend;
pub mod init;
pub mod locaize;
pub mod metrc;
pub mod refs;
pub mod setting;
pub mod spec;
pub mod target;
use derive_more::{Display, From};
use serde::Serializer;
use serde_derive::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

#[derive(Clone, Debug, Serialize, Deserialize, Display, PartialEq, Eq, Hash)]
pub enum CpuArch {
    #[display("x86")]
    X86,
    #[display("arm")]
    Arm,
}

impl FromStr for CpuArch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x86" => Ok(Self::X86),
            "arm" => Ok(Self::Arm),
            _ => Err(s.to_string()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Display, PartialEq, Eq, Hash)]
pub enum OsCPE {
    #[display("mac14")]
    MAC14,
    #[display("win10")]
    WIN10,
    #[display("ubt22")]
    UBT22,
    #[display("cos7")]
    COS7,
}

impl FromStr for OsCPE {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mac14" => Ok(Self::MAC14),
            "win10" => Ok(Self::WIN10),
            "ubt22" => Ok(Self::UBT22),
            "cos7" => Ok(Self::COS7),
            _ => Err(s.to_string()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Display, PartialEq, Eq, Hash)]
pub enum RunSPC {
    #[display("host")]
    Host,
    #[display("k8s")]
    K8S,
}
impl FromStr for RunSPC {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "host" => Ok(Self::Host),
            "k8s" => Ok(Self::K8S),
            _ => Err(s.to_string()),
        }
    }
}

#[derive(Clone, Debug, From, PartialEq, Eq, Hash)]
pub struct TargetNode {
    arch: CpuArch,
    os: OsCPE,
    spc: RunSPC,
}
impl TargetNode {
    pub fn arm_mac14_host() -> Self {
        Self {
            arch: CpuArch::Arm,
            os: OsCPE::MAC14,
            spc: RunSPC::Host,
        }
    }
    pub fn x86_ubt22_host() -> Self {
        Self {
            arch: CpuArch::X86,
            os: OsCPE::UBT22,
            spc: RunSPC::Host,
        }
    }
    pub fn x86_ubt22_k8s() -> Self {
        Self {
            arch: CpuArch::X86,
            os: OsCPE::UBT22,
            spc: RunSPC::K8S,
        }
    }
}

// 紧凑的序列化实现
impl serde::Serialize for TargetNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 格式化为 "arch-os-spc" 字符串
        let s = format!("{}-{}-{}", self.arch, self.os, self.spc);
        serializer.serialize_str(&s)
    }
}

// 对应的反序列化实现
impl<'de> serde::Deserialize<'de> for TargetNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl TargetNode {
    pub fn new(arch: CpuArch, os: OsCPE, spc: RunSPC) -> Self {
        Self { arch, os, spc }
    }
}

impl Display for TargetNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}", self.arch, self.os, self.spc)
    }
}
impl FromStr for TargetNode {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = value.split('-').collect();
        if parts.len() != 3 {
            return Err(value.into());
        }

        let arch = CpuArch::from_str(parts[0])?;
        let os = OsCPE::from_str(parts[1])?;
        let spc = RunSPC::from_str(parts[2])?;

        Ok(TargetNode { arch, os, spc })
    }
}
