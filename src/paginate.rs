#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageInfo {
    pub index: usize,
    pub total_pages: usize,
}

pub fn paginate<T>(items: &[T], size: usize) -> Vec<&[T]> {
    if size == 0 {
        return Vec::new();
    }
    items.chunks(size).collect()
}

pub fn neighbors(info: PageInfo) -> (Option<usize>, Option<usize>) {
    let prev = if info.index > 0 {
        Some(info.index - 1)
    } else {
        None
    };
    let next = if info.index + 1 < info.total_pages {
        Some(info.index + 1)
    } else {
        None
    };
    (prev, next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_items_zero_pages() {
        let items: [u8; 0] = [];
        let pages = paginate(&items, 10);
        assert!(pages.is_empty());
    }

    #[test]
    fn twenty_three_items_size_ten() {
        let items: Vec<u32> = (0..23).collect();
        let pages = paginate(&items, 10);
        assert_eq!(pages.len(), 3);
        assert_eq!(pages[0].len(), 10);
        assert_eq!(pages[1].len(), 10);
        assert_eq!(pages[2].len(), 3);
    }
}
