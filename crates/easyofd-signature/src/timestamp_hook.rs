//! 时间戳 Hook。
//!
//! 对应 Java: `org.ofdrw.sign.timestamp.TimeStampHook`

/// 时间戳 Hook 接口。
///
/// 对应 Java: `org.ofdrw.sign.timestamp.TimeStampHook`
///
/// 函数式接口，在签名完成后调用，用于获取签名值的时间戳。
/// 返回时间戳的 DER 编码字节。
pub trait TimeStampHook: Send + Sync {
    /// 对签名值获取时间戳。
    ///
    /// # 参数
    ///
    /// - `signature`：签章签名值字节
    ///
    /// # 返回
    ///
    /// 时间戳的 DER 编码字节。
    fn apply(&self, signature: &[u8]) -> Vec<u8>;
}

/// 闭包形式的时间戳 Hook。
///
/// 对应 Java: `org.ofdrw.sign.timestamp.TimeStampHook` 的 lambda 实现。
///
/// # 示例
///
/// ```
/// use easyofd_signature::timestamp_hook::{ClosureTimeStampHook, TimeStampHook};
///
/// let hook = ClosureTimeStampHook::new(|sig| {
///     // 模拟时间戳生成
///     vec![0x01, 0x02, 0x03]
/// });
/// let ts = hook.apply(&[0xAA, 0xBB]);
/// assert_eq!(ts, vec![0x01, 0x02, 0x03]);
/// ```
pub struct ClosureTimeStampHook<F>
where
    F: Fn(&[u8]) -> Vec<u8> + Send + Sync,
{
    func: F,
}

impl<F> ClosureTimeStampHook<F>
where
    F: Fn(&[u8]) -> Vec<u8> + Send + Sync,
{
    /// 从闭包创建时间戳 Hook。
    #[must_use]
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F> TimeStampHook for ClosureTimeStampHook<F>
where
    F: Fn(&[u8]) -> Vec<u8> + Send + Sync,
{
    fn apply(&self, signature: &[u8]) -> Vec<u8> {
        (self.func)(signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closure_hook_returns_value() {
        let hook = ClosureTimeStampHook::new(|_sig| vec![0xDE, 0xAD]);
        assert_eq!(hook.apply(&[0x01]), vec![0xDE, 0xAD]);
    }

    #[test]
    fn closure_hook_receives_signature() {
        #[allow(clippy::redundant_closure_for_method_calls)]
        let hook = ClosureTimeStampHook::new(|sig| sig.to_vec());
        let input = vec![0x01, 0x02, 0x03];
        assert_eq!(hook.apply(&input), input);
    }

    #[test]
    fn trait_object_works() {
        let hook: Box<dyn TimeStampHook> = Box::new(ClosureTimeStampHook::new(|_| vec![0xFF]));
        assert_eq!(hook.apply(&[]), vec![0xFF]);
    }
}
