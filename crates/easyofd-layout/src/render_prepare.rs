//! 渲染准备接口。
//!
//! 对应 Java: org.ofdrw.layout.RenderPrepare

/// 渲染准备接口，在渲染前对元素进行预处理。
///
/// 对应 Java: ofdrw layout RenderPrepare（interface）。
pub trait RenderPrepare {
    /// 渲染准备，返回 `true` 表示准备完成可以渲染，`false` 表示跳过。
    fn prepare(&mut self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPrepare {
        ready: bool,
    }

    impl RenderPrepare for TestPrepare {
        fn prepare(&mut self) -> bool {
            self.ready = true;
            self.ready
        }
    }

    #[test]
    fn test_render_prepare() {
        let mut p = TestPrepare { ready: false };
        assert!(!p.ready);
        let result = p.prepare();
        assert!(result);
        assert!(p.ready);
    }
}
