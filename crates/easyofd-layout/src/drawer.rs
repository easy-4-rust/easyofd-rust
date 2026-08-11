//! Canvas 内容绘制器接口。
//!
//! 对应 Java: org.ofdrw.layout.element.canvas.Drawer
//!
//! 用于绘制 Canvas 中的实际内容。在 Rust 中以 trait 表达函数式接口语义。

/// Canvas 内容绘制器。
///
/// 对应 Java: ofdrw layout canvas Drawer（`@FunctionalInterface`）。
///
/// 实现此 trait 的类型可以被 Canvas 使用来绘制自定义内容。
pub trait Drawer {
    /// 绘制错误类型。
    type Error;

    /// 执行绘制操作。
    ///
    /// # Errors
    ///
    /// 实现方应在绘制失败时返回错误（如 IO 异常、图片读取异常等）。
    fn draw(&self) -> Result<(), Self::Error>;
}

/// 闭包形式的绘制器（对应 Java: Drawer 的 lambda 用法）。
///
/// 允许将闭包直接作为 `Drawer` 使用。
pub struct FnDrawer<F>(pub F)
where
    F: Fn() -> Result<(), Box<dyn std::error::Error>>;

impl<F> Drawer for FnDrawer<F>
where
    F: Fn() -> Result<(), Box<dyn std::error::Error>>,
{
    type Error = Box<dyn std::error::Error>;

    fn draw(&self) -> Result<(), Self::Error> {
        (self.0)()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDrawer {
        called: std::cell::Cell<bool>,
    }

    impl TestDrawer {
        fn new() -> Self {
            Self {
                called: std::cell::Cell::new(false),
            }
        }
    }

    impl Drawer for TestDrawer {
        type Error = String;

        fn draw(&self) -> Result<(), Self::Error> {
            self.called.set(true);
            Ok(())
        }
    }

    #[test]
    fn test_drawer_trait() {
        let drawer = TestDrawer::new();
        assert!(!drawer.called.get());
        drawer.draw().unwrap();
        assert!(drawer.called.get());
    }

    #[test]
    fn test_fn_drawer() {
        let drawer = FnDrawer(|| Ok(()));
        assert!(drawer.draw().is_ok());
    }

    #[test]
    fn test_fn_drawer_error() {
        let drawer = FnDrawer(|| Err("draw error".into()));
        assert!(drawer.draw().is_err());
    }
}
