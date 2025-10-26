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

/// 应用窗口排序逻辑
pub fn apply_window_sorting(
    windows: &mut [crate::types::WindowInfo], 
    sort_pid: &SortOrder, 
    sort_position: &PositionSort
) {
    windows.sort_by(|a, b| {
        let mut cmp = std::cmp::Ordering::Equal;
        
        // 首先按位置排序（如果指定了）
        if !matches!(sort_position.x_order, SortOrder::None) || !matches!(sort_position.y_order, SortOrder::None) {
            // X 坐标排序
            if !matches!(sort_position.x_order, SortOrder::None) {
                cmp = a.rect.x.cmp(&b.rect.x);
                if let SortOrder::Descending = sort_position.x_order {
                    cmp = cmp.reverse();
                }
            }
            
            // 如果 X 坐标相同，则按 Y 坐标排序
            if cmp == std::cmp::Ordering::Equal && !matches!(sort_position.y_order, SortOrder::None) {
                cmp = a.rect.y.cmp(&b.rect.y);
                if let SortOrder::Descending = sort_position.y_order {
                    cmp = cmp.reverse();
                }
            }
        }
        
        // 如果位置相同或未指定位置排序，则按 PID 排序
        if cmp == std::cmp::Ordering::Equal {
            match sort_pid {
                SortOrder::Ascending => cmp = a.pid.cmp(&b.pid),
                SortOrder::Descending => cmp = b.pid.cmp(&a.pid),
                SortOrder::None => {} // 保持相等
            }
        }
        
        cmp
    });
}

/// 对 WindowHandle 进行排序（当没有位置信息时使用）
pub fn apply_window_handle_sorting(
    windows: &mut [crate::platform::WindowHandle],  // 使用正确的路径
    sort_pid: &SortOrder, 
    sort_position: &PositionSort
) {
    // 由于 WindowHandle 不包含位置信息，我们按标题和PID排序作为替代
    windows.sort_by(|a, b| {
        let mut cmp = std::cmp::Ordering::Equal;
        
        // 按标题排序作为位置排序的替代
        if !matches!(sort_position.x_order, SortOrder::None) {
            cmp = a.title.cmp(&b.title);
            if let SortOrder::Descending = sort_position.x_order {
                cmp = cmp.reverse();
            }
        }
        
        // 如果标题相同，按PID排序
        if cmp == std::cmp::Ordering::Equal {
            match sort_pid {
                SortOrder::Ascending => cmp = a.pid.cmp(&b.pid),
                SortOrder::Descending => cmp = b.pid.cmp(&a.pid),
                SortOrder::None => {} // 保持相等
            }
        }
        
        cmp
    });
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
}