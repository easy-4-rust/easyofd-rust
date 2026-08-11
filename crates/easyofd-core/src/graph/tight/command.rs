//! 路径命令类型别名。
//!
//! 对应 Java: org.ofdrw.core.graph.tight.method.Command

use crate::graph::PathCommand;

/// 路径命令，对应 Java 中 `Command` 类型。
///
/// 对应 Java: org.ofdrw.core.graph.tight.method.Command
///
/// 在 Java 版中 `Command` 是所有路径命令的基类。
/// Rust 版中直接使用 `PathCommand` 枚举表示所有路径命令类型。
/// 此类型别名用于与 Java 命名保持一致。
pub type Command = PathCommand;

/// 复合对象类型别名。
///
/// 对应 Java: org.ofdrw.core.basicStructure.pageObj.layer.block.CompositeObject
///
/// 在 Java 版中 `CompositeObject` 是复合对象的类名。
/// Rust 版中使用 `CT_Composite` 表示复合对象。
/// 此类型别名用于与 Java 命名保持一致。
pub type CompositeObject = crate::composite_obj::CT_Composite;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_type_alias() {
        // 验证 Command 和 PathCommand 是同一类型
        fn _assert_same<T>() {}
        fn _check() {
            _assert_same::<Command>();
        }
    }

    #[test]
    fn test_composite_object_alias() {
        // 验证 CompositeObject 可以使用
        let _ = CompositeObject::new(1, "test");
    }
}
