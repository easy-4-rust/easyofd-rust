//! 绘制参数资源。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.res.resources.DrawParams

/// 绘制参数资源。
///
/// 对应 Java: org.ofdrw.core.basicStructure.res.resources.DrawParams
#[derive(Debug, Clone, Default)]
pub struct DrawParams {
    /// 绘制参数列表。
    pub params: Vec<String>,
}

impl DrawParams {
    /// 创建空绘制参数。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加参数。
    pub fn add_param(&mut self, param: impl Into<String>) {
        self.params.push(param.into());
    }

    /// 获取参数数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_params_new() {
        let dp = DrawParams::new();
        assert!(dp.is_empty());
    }

    #[test]
    fn draw_params_add() {
        let mut dp = DrawParams::new();
        dp.add_param("<ofd:DrawParam/>");
        assert_eq!(dp.len(), 1);
    }
}
