use crate::core_new::DataType;
use crate::core_new::NodeResult;
impl From<DataType> for isize{
    fn from(data: DataType) -> Self {
        match data {
            DataType::Usize(u) => u as isize,
            DataType::Isize(i) => i,
            DataType::Bool(b) => b as isize,
            _ => 0,
        }
    }
}

impl From<DataType> for usize{
    fn from(data: DataType) -> Self{
        match data {
            DataType::Usize(u) => u,
            DataType::Isize(i) => i as usize,
            DataType::Bool(b) => b as usize,
            _ => 0,
        }
    }
}

impl From<DataType> for bool{
    fn from(data: DataType) -> Self{
        match data {
            DataType::F32(f32) => f32 > 0.0,
            DataType::Usize(u) => u > 0,
            DataType::Isize(i) => i > 0,
            DataType::Bool(b) => b,
            _ => false
        }
    }
}

impl From<DataType> for f32{
    fn from(data: DataType) -> Self{
        match data {
            DataType::F32(f32) => f32,
            DataType::Usize(u) => u as f32,
            DataType::Isize(i) => i as f32,
            _ => 0.0,
        }
    }
}

impl From<DataType> for String{
    fn from(data: DataType) -> Self{
        match data {
            DataType::F32(f32) => f32.to_string(),
            DataType::String(str) => str,
            DataType::Usize(u) => u.to_string(),
            DataType::Isize(i) => i.to_string(),
            DataType::Bool(b) => b.to_string(),
        }
    }
}

impl Into<DataType> for isize{
    fn into(self) -> DataType {
        DataType::Isize(self)
    }
}

impl Into<DataType> for usize{
    fn into(self) -> DataType {
        DataType::Usize(self)
    }
}

impl Into<DataType> for String{
    fn into(self) -> DataType {
        DataType::String(self)
    }
}

impl Into<DataType> for f32{
    fn into(self) -> DataType {
        DataType::F32(self)
    }
}

impl Into<DataType> for bool{
    fn into(self) -> DataType {
        DataType::Bool(self)
    }
}

impl From<usize> for NodeResult{
    fn from(u: usize) -> Self {
        NodeResult::NodeID(u)
    }
}

impl From<&str> for NodeResult{
    fn from(str:&str) -> Self {
        NodeResult::NodeName(str.to_string())
    }
}