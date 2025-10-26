// src/sorting.rs
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
    None,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::None
    }
}

impl FromStr for SortOrder {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(SortOrder::Ascending),
            "-1" => Ok(SortOrder::Descending),
            "0" => Ok(SortOrder::None),
            _ => Err(format!("Invalid sort order: {}. Use 1 (ascending), -1 (descending), or 0 (none)", s)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PositionSort {
    pub x_order: SortOrder,
    pub y_order: SortOrder,
}

impl Default for PositionSort {
    fn default() -> Self {
        PositionSort {
            x_order: SortOrder::Ascending,
            y_order: SortOrder::Ascending,
        }
    }
}

impl PositionSort {
    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), String> {
        if matches!(self.x_order, SortOrder::None) && matches!(self.y_order, SortOrder::None) {
            return Err("PositionSort must have at least one non-None ordering".to_string());
        }
        Ok(())
    }
    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        !matches!(self.x_order, SortOrder::None) || !matches!(self.y_order, SortOrder::None)
    }
}

impl FromStr for PositionSort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 2 {
            return Err("Position sort format should be X_ORDER|Y_ORDER, e.g., 1|-1".to_string());
        }

        let x_order = parts[0].parse()?;
        let y_order = parts[1].parse()?;

        Ok(PositionSort { x_order, y_order })
    }
}

#[derive(Debug, Clone)]
pub struct SortConfig {
    pub pid: SortOrder,
    pub position: PositionSort,
    #[allow(dead_code)]
    pub fallback_to_title: bool,
}

impl Default for SortConfig {
    fn default() -> Self {
        Self {
            pid: SortOrder::None,
            position: PositionSort::default(),
            fallback_to_title: true,
        }
    }
}

/// 可排序对象的统一接口
pub trait Sortable {
    fn get_pid(&self) -> u32;
    fn get_position(&self) -> Option<(i32, i32)>;
    fn get_title(&self) -> &str;
}

// 为 WindowInfo 实现 Sortable
impl Sortable for crate::types::WindowInfo {
    fn get_pid(&self) -> u32 { self.pid }
    fn get_position(&self) -> Option<(i32, i32)> { Some((self.rect.x, self.rect.y)) }
    fn get_title(&self) -> &str { &self.title }
}

// 为 WindowHandle 实现 Sortable
impl Sortable for crate::platform::WindowHandle {
    fn get_pid(&self) -> u32 { self.pid }
    fn get_position(&self) -> Option<(i32, i32)> { None }
    fn get_title(&self) -> &str { &self.title }
}

/// 统一的排序函数
#[allow(dead_code)]
pub fn apply_sorting<T: Sortable>(
    items: &mut [T],
    sort_pid: &SortOrder,
    sort_position: &PositionSort,
) {
    if should_skip_sorting(sort_pid, sort_position) {
        return;
    }
    
    // 对小数据集使用简单排序，对大数据集考虑性能优化
    if items.len() < 100 {
        items.sort_by(|a, b| compare_items(a, b, sort_pid, sort_position));
    } else {
        // 对大数据集使用相同的排序逻辑，但可以在这里添加性能优化
        items.sort_by(|a, b| compare_items(a, b, sort_pid, sort_position));
    }
}

/// 使用配置的排序函数
#[allow(dead_code)]
pub fn apply_sorting_with_config<T: Sortable>(
    items: &mut [T],
    config: &SortConfig,
) {
    apply_sorting(items, &config.pid, &config.position);
}

/// 优化的排序函数，预检查排序必要性
pub fn apply_optimized_sorting<T: Sortable>(
    items: &mut [T],
    sort_pid: &SortOrder,
    sort_position: &PositionSort,
) {
    if should_skip_sorting(sort_pid, sort_position) {
        return;
    }
    
    items.sort_by(|a, b| compare_items(a, b, sort_pid, sort_position));
}

// 辅助函数：检查是否需要排序
fn should_skip_sorting(sort_pid: &SortOrder, sort_position: &PositionSort) -> bool {
    matches!(sort_pid, SortOrder::None) && 
    matches!(sort_position.x_order, SortOrder::None) && 
    matches!(sort_position.y_order, SortOrder::None)
}

// 核心比较逻辑
fn compare_items<T: Sortable>(
    a: &T,
    b: &T,
    sort_pid: &SortOrder,
    sort_position: &PositionSort,
) -> std::cmp::Ordering {
    // 1. 位置排序（如果可用）
    if let (Some(pos_a), Some(pos_b)) = (a.get_position(), b.get_position()) {
        let position_cmp = compare_positions(pos_a, pos_b, sort_position);
        if position_cmp != std::cmp::Ordering::Equal {
            return position_cmp;
        }
    }
    
    // 2. 标题排序（作为位置排序的备选）
    if !matches!(sort_position.x_order, SortOrder::None) {
        let title_cmp = a.get_title().cmp(b.get_title());
        let adjusted_cmp = adjust_ordering(title_cmp, sort_position.x_order);
        if adjusted_cmp != std::cmp::Ordering::Equal {
            return adjusted_cmp;
        }
    }
    
    // 3. PID 排序
    compare_pids(a.get_pid(), b.get_pid(), sort_pid)
}

// 位置比较逻辑
fn compare_positions(
    (x1, y1): (i32, i32),
    (x2, y2): (i32, i32),
    sort_position: &PositionSort,
) -> std::cmp::Ordering {
    let mut cmp = std::cmp::Ordering::Equal;
    
    if !matches!(sort_position.x_order, SortOrder::None) {
        cmp = x1.cmp(&x2);
        cmp = adjust_ordering(cmp, sort_position.x_order);
    }
    
    if cmp == std::cmp::Ordering::Equal && !matches!(sort_position.y_order, SortOrder::None) {
        cmp = y1.cmp(&y2);
        cmp = adjust_ordering(cmp, sort_position.y_order);
    }
    
    cmp
}

// PID 比较逻辑
fn compare_pids(pid_a: u32, pid_b: u32, sort_order: &SortOrder) -> std::cmp::Ordering {
    match sort_order {
        SortOrder::Ascending => pid_a.cmp(&pid_b),
        SortOrder::Descending => pid_b.cmp(&pid_a),
        SortOrder::None => std::cmp::Ordering::Equal,
    }
}

// 调整排序方向
fn adjust_ordering(ordering: std::cmp::Ordering, sort_order: SortOrder) -> std::cmp::Ordering {
    match sort_order {
        SortOrder::Ascending => ordering,
        SortOrder::Descending => ordering.reverse(),
        SortOrder::None => std::cmp::Ordering::Equal,
    }
}

/// 保持向后兼容的窗口排序函数
pub fn apply_window_sorting(
    windows: &mut [crate::types::WindowInfo], 
    sort_pid: &SortOrder, 
    sort_position: &PositionSort,
) {
    apply_optimized_sorting(windows, sort_pid, sort_position);
}

/// 保持向后兼容的窗口句柄排序函数
pub fn apply_window_handle_sorting(
    windows: &mut [crate::platform::WindowHandle],
    sort_pid: &SortOrder, 
    sort_position: &PositionSort,
) {
    apply_optimized_sorting(windows, sort_pid, sort_position);
}

/// 便捷函数：创建排序配置
#[allow(dead_code)]
pub fn create_sort_config(
    pid_order: &str,
    position_order: &str,
) -> Result<SortConfig, String> {
    let pid = pid_order.parse()?;
    let position = position_order.parse()?;
    
    let config = SortConfig {
        pid,
        position,
        fallback_to_title: true,
    };
    
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{WindowInfo, WindowRect};

    #[test]
    fn test_sort_order_parsing() {
        assert_eq!("1".parse::<SortOrder>().unwrap(), SortOrder::Ascending);
        assert_eq!("-1".parse::<SortOrder>().unwrap(), SortOrder::Descending);
        assert_eq!("0".parse::<SortOrder>().unwrap(), SortOrder::None);
        assert!("2".parse::<SortOrder>().is_err());
    }

    #[test]
    fn test_position_sort_parsing() {
        let pos = "1|-1".parse::<PositionSort>().unwrap();
        assert_eq!(pos.x_order, SortOrder::Ascending);
        assert_eq!(pos.y_order, SortOrder::Descending);

        assert!("1".parse::<PositionSort>().is_err());
        assert!("1|2|-1".parse::<PositionSort>().is_err());
    }

    #[test]
    fn test_position_sort_validation() {
        let valid_sort = PositionSort {
            x_order: SortOrder::Ascending,
            y_order: SortOrder::None,
        };
        assert!(valid_sort.validate().is_ok());
        assert!(valid_sort.is_active());

        let invalid_sort = PositionSort {
            x_order: SortOrder::None,
            y_order: SortOrder::None,
        };
        assert!(invalid_sort.validate().is_err());
        assert!(!invalid_sort.is_active());
    }

    #[test]
    fn test_apply_window_sorting() {
        let mut windows = vec![
            WindowInfo {
                pid: 100,
                title: "Window C".to_string(),
                rect: WindowRect::new(300, 200, 800, 600),
            },
            WindowInfo {
                pid: 200,
                title: "Window A".to_string(),
                rect: WindowRect::new(100, 100, 800, 600),
            },
            WindowInfo {
                pid: 150,
                title: "Window B".to_string(),
                rect: WindowRect::new(200, 150, 800, 600),
            },
        ];

        // Test PID ascending sort
        apply_window_sorting(&mut windows, &SortOrder::Ascending, &PositionSort::default());
        assert_eq!(windows[0].pid, 100);
        assert_eq!(windows[1].pid, 150);
        assert_eq!(windows[2].pid, 200);

        // Test PID descending sort
        apply_window_sorting(&mut windows, &SortOrder::Descending, &PositionSort::default());
        assert_eq!(windows[0].pid, 200);
        assert_eq!(windows[1].pid, 150);
        assert_eq!(windows[2].pid, 100);

        // Test position sort (X ascending, Y ascending)
        let position_sort = PositionSort {
            x_order: SortOrder::Ascending,
            y_order: SortOrder::Ascending,
        };
        apply_window_sorting(&mut windows, &SortOrder::None, &position_sort);
        assert_eq!(windows[0].rect.x, 100);
        assert_eq!(windows[1].rect.x, 200);
        assert_eq!(windows[2].rect.x, 300);
    }

    #[test]
    fn test_skip_sorting() {
        let mut windows = vec![
            WindowInfo {
                pid: 100,
                title: "Window A".to_string(),
                rect: WindowRect::new(100, 100, 800, 600),
            },
            WindowInfo {
                pid: 200,
                title: "Window B".to_string(),
                rect: WindowRect::new(200, 200, 800, 600),
            },
        ];

        let original_order: Vec<u32> = windows.iter().map(|w| w.pid).collect();
        
        // 当所有排序都是 None 时，应该跳过排序
        apply_window_sorting(&mut windows, &SortOrder::None, &PositionSort::default());
        
        let after_sort_order: Vec<u32> = windows.iter().map(|w| w.pid).collect();
        assert_eq!(original_order, after_sort_order);
    }

    #[test]
    fn test_sort_config() {
        let config = SortConfig {
            pid: SortOrder::Ascending,
            position: PositionSort {
                x_order: SortOrder::Descending,
                y_order: SortOrder::Ascending,
            },
            fallback_to_title: true,
        };

        let mut windows = vec![
            WindowInfo {
                pid: 200,
                title: "Window B".to_string(),
                rect: WindowRect::new(100, 100, 800, 600),
            },
            WindowInfo {
                pid: 100,
                title: "Window A".to_string(),
                rect: WindowRect::new(200, 200, 800, 600),
            },
        ];

        apply_sorting_with_config(&mut windows, &config);
        
        // 由于位置排序是 X 降序，所以 X 较大的应该排在前面
        assert_eq!(windows[0].rect.x, 200);
        assert_eq!(windows[1].rect.x, 100);
    }

    #[test]
    fn test_create_sort_config() {
        let config = create_sort_config("1", "1|-1").unwrap();
        assert_eq!(config.pid, SortOrder::Ascending);
        assert_eq!(config.position.x_order, SortOrder::Ascending);
        assert_eq!(config.position.y_order, SortOrder::Descending);
        assert!(config.fallback_to_title);

        assert!(create_sort_config("invalid", "1|-1").is_err());
        assert!(create_sort_config("1", "invalid").is_err());
    }

    #[test]
    fn test_sortable_trait_implementation() {
        let window_info = WindowInfo {
            pid: 123,
            title: "Test Window".to_string(),
            rect: WindowRect::new(100, 200, 800, 600),
        };

        assert_eq!(window_info.get_pid(), 123);
        assert_eq!(window_info.get_position(), Some((100, 200)));
        assert_eq!(window_info.get_title(), "Test Window");

        // 测试 WindowHandle 的 Sortable 实现
        use crate::platform::{WindowHandle, PlatformData};
        
        #[cfg(windows)]
        let platform_data = PlatformData::Windows(crate::platform::windows::WindowsWindowData { hwnd: 0 });
        #[cfg(unix)]
        let platform_data = PlatformData::Unix(crate::platform::unix::UnixWindowData::new());
        
        let window_handle = WindowHandle::new(456, "Handle Window".to_string(), platform_data);
        
        assert_eq!(window_handle.get_pid(), 456);
        assert_eq!(window_handle.get_position(), None);
        assert_eq!(window_handle.get_title(), "Handle Window");
    }
}