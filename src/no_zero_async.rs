use std::future::Future;
use std::io::Write;
use std::vec;

use bytes::Bytes;

/// # async-trait demo
/// ```
/// #[async_trait]
/// trait KvIterator {
///     async fn next(&mut self) -> Option<(&[u8], &[u8])>;
/// }
/// ```
/// # Expand
/// pub trait KvIterator {
///   fn next(&mut self) -> Box<dyn Future<Output = Option<(&[u8],&[u8])>>>;
/// }
///
/// 上面使用 async_trait 会带下性能开销
/// 1. vtable 动态调度的开销 dyn Future
/// 2. Box 会在堆上进行分配，在进行iter 的时候每调用一次 next 都会在堆上分配

pub trait KvIterator {
    type NextFuture<'a>: Future<Output = Option<(&'a [u8], &'a [u8])>>
    where
        Self: 'a;

    fn next(&mut self) -> Self::NextFuture<'_>;
}

pub struct TestIterator {
    idx: usize,
    to_idx: usize,
    key: Vec<u8>,
    value: Vec<u8>,
}

impl KvIterator for TestIterator {
    type NextFuture<'a>
    where
        Self: 'a,
    = impl Future<Output = Option<(&'a [u8], &'a [u8])>>;

    fn next(&mut self) -> Self::NextFuture<'_> {
        async move {
            if self.idx >= self.to_idx {
                return None;
            }

            self.key.clear();
            write!(&mut self.key, "key_{:05}", self.idx).unwrap();

            self.value.clear();
            write!(&mut self.value, "value_{:05}", self.idx).unwrap();

            self.idx += 1;
            Some((&self.key[..], &self.value[..]))
        }
    }
}

impl TestIterator {
    pub fn new(from_idx: usize, to_idx: usize) -> Self {
        Self {
            idx: from_idx,
            to_idx,
            key: Vec::new(),
            value: Vec::new(),
        }
    }
}

pub struct ConcatIterator<Iter: KvIterator> {
    iters: Vec<Iter>,
    key: Vec<u8>,
    value: Vec<u8>,
    current_idx: usize,
}

impl<Iter: KvIterator> ConcatIterator<Iter> {
    pub fn new(iters: Vec<Iter>) -> Self {
        Self {
            iters,
            current_idx: 0,
            key: Vec::new(),
            value: Vec::new(),
        }
    }
}

impl<Iter: KvIterator> KvIterator for ConcatIterator<Iter> {
    type NextFuture<'a>
    where
        Self: 'a,
    = impl Future<Output = Option<(&'a [u8], &'a [u8])>>;

    fn next(&mut self) -> Self::NextFuture<'_> {
        async move {
            loop {
                if self.current_idx >= self.iters.len() {
                    return None;
                }
                let iter = &mut self.iters[self.current_idx];
                match iter.next().await {
                    Some((key, value)) => {
                        self.key.clear();
                        self.key.extend_from_slice(key);
                        self.value.clear();
                        self.value.extend_from_slice(value);
                    }
                    None => {
                        self.current_idx += 1;
                    }
                }
            }
        }
    }
}

/// 使用上面的方式，在结构体里存放了key，value 来暂存，但是每次 next 的时候就会多一个拷贝
/// 重构 iterator trait. .next 只移动迭代器的位置，key，value 返回内容
trait KkIterator {
    type KvIteratorNextFuture<'a>: Future<Output = ()>
    where
        Self: 'a;
    fn next(&mut self) -> Self::KvIteratorNextFuture<'_>;
    fn key(&self) -> &[u8];
    fn value(&self) -> &[u8];
    fn is_valid(&self) -> bool;
}

#[tokio::test]
async fn test_iterator() {
    let mut iter = TestIterator::new(0, 10);
    while let Some((key, value)) = iter.next().await {
        println!(
            "{:?}-{:?}",
            Bytes::copy_from_slice(key),
            Bytes::copy_from_slice(value),
        )
    }
}
