// src/utils.rs
// #[allow(unused_imports)]
// use std::str::FromStr;

/// 解析索引字符串，如 "1,2,3" -> [1, 2, 3]
pub fn parse_indices(index_str: &str, max_index: usize) -> Vec<usize> {
    if index_str.trim().is_empty() {
        return Vec::new();
    }

    index_str
        .split(',')
        .filter_map(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                trimmed.parse::<usize>().ok()
            }
        })
        .filter(|&idx| idx >= 1 && idx <= max_index)
        .collect()
}

/// 验证位置参数组合的合理性
pub fn validate_position_parameters(
    position: &Option<String>,
    layout: &Option<String>,
    x_start: &Option<String>,
    y_start: &Option<String>,
    x_step: &Option<String>,
    y_step: &Option<String>,
) -> Result<(), String> {
    let has_single_position = position.is_some();
    let has_layout = layout.as_ref().map_or(false, |s| !s.trim().is_empty());
    let has_grid = x_start.is_some() || y_start.is_some() || x_step.is_some() || y_step.is_some();

    let method_count = [has_single_position, has_layout, has_grid]
        .iter()
        .filter(|&&b| b)
        .count();

    if method_count == 0 {
        return Err("No position method specified. Use --position, --layout, or --x-start/--y-start with steps".to_string());
    }

    if method_count > 1 {
        return Err("Multiple position methods specified. Use only one of --position, --layout, or grid parameters".to_string());
    }

    Ok(())
}

/// 计算窗口位置列表
pub fn calculate_positions(
    window_count: usize,
    position: &Option<String>,
    layout: &str,
    x_start: &Option<String>,
    y_start: &Option<String>,
    x_step: &Option<String>,
    y_step: &Option<String>,
) -> Result<Vec<(i32, i32)>, String> {
    if let Some(pos_str) = position {
        // 单一位置模式
        let (x, y) = parse_position(pos_str)?;
        Ok(vec![(x, y); window_count])
    } else if !layout.trim().is_empty() {
        // 布局模式
        parse_layout(layout, window_count)
    } else if x_start.is_some() || y_start.is_some() {
        // 网格模式
        let x_start = x_start.as_ref().and_then(|s| s.parse().ok()).unwrap_or(0);
        let y_start = y_start.as_ref().and_then(|s| s.parse().ok()).unwrap_or(0);
        let x_step = x_step.as_ref().and_then(|s| s.parse().ok()).unwrap_or(100);
        let y_step = y_step.as_ref().and_then(|s| s.parse().ok()).unwrap_or(100);

        let mut positions = Vec::new();
        
        for i in 0..window_count { 
            let x = x_start + (i as i32) * x_step;
            let y = y_start + (i as i32) * y_step;
            
            positions.push((x, y));
        }

        Ok(positions)
    } else {
        Err("No valid position configuration found".to_string())
    }
}

/// 解析单一位置字符串 "X,Y" -> (x, y)
pub fn parse_position(position_str: &str) -> Result<(i32, i32), String> {
    let parts: Vec<&str> = position_str.split(',').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid position format: {}. Expected 'X,Y'", position_str));
    }

    let x = parts[0].trim().parse().map_err(|_| format!("Invalid X coordinate: {}", parts[0]))?;
    let y = parts[1].trim().parse().map_err(|_| format!("Invalid Y coordinate: {}", parts[1]))?;

    Ok((x, y))
}

/// 解析布局字符串 "X1,Y1,X2,Y2,..." -> [(x1, y1), (x2, y2), ...]
pub fn parse_layout(layout_str: &str, window_count: usize) -> Result<Vec<(i32, i32)>, String> {
    let coords: Vec<&str> = layout_str.split(',').collect();
    
    if coords.len() % 2 != 0 {
        return Err(format!("Layout must have even number of coordinates, got {}", coords.len()));
    }

    let mut positions = Vec::new();
    for chunk in coords.chunks(2) {
        let x = chunk[0].trim().parse()
            .map_err(|_| format!("Invalid X coordinate in layout: {}", chunk[0]))?;
        let y = chunk[1].trim().parse()
            .map_err(|_| format!("Invalid Y coordinate in layout: {}", chunk[1]))?;
        positions.push((x, y));
    }

    if positions.len() < window_count {
        return Err(format!("Not enough positions in layout (need {}, got {})", window_count, positions.len()));
    }

    // 如果提供的位置多于窗口数量，只取需要的数量
    positions.truncate(window_count);
    Ok(positions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_indices() {
        assert_eq!(parse_indices("", 5), vec![]);
        assert_eq!(parse_indices("1,2,3", 5), vec![1, 2, 3]);
        assert_eq!(parse_indices("1, 2, 3", 5), vec![1, 2, 3]);
        assert_eq!(parse_indices("1,6,3", 5), vec![1, 3]); // 6 is out of bounds
        assert_eq!(parse_indices("1,,3", 5), vec![1, 3]); // empty element is skipped
    }

    #[test]
    fn test_parse_position() {
        assert_eq!(parse_position("100,200").unwrap(), (100, 200));
        assert_eq!(parse_position(" 100 , 200 ").unwrap(), (100, 200));
        assert!(parse_position("100").is_err());
        assert!(parse_position("100,200,300").is_err());
        assert!(parse_position("abc,def").is_err());
    }

    #[test]
    fn test_parse_layout() {
        assert_eq!(parse_layout("100,200,150,250", 2).unwrap(), vec![(100, 200), (150, 250)]);
        assert_eq!(parse_layout("100,200,150,250,200,300", 2).unwrap(), vec![(100, 200), (150, 250)]);
        assert!(parse_layout("100,200,150", 2).is_err()); // odd number
        assert!(parse_layout("100,200", 2).is_err()); // not enough positions
    }

    #[test]
    fn test_validate_position_parameters() {
        // 测试无参数
        assert!(validate_position_parameters(&None, &None, &None, &None, &None, &None).is_err());
        
        // 测试单一位置参数
        assert!(validate_position_parameters(&Some("100,200".to_string()), &None, &None, &None, &None, &None).is_ok());
        
        // 测试布局参数
        assert!(validate_position_parameters(&None, &Some("100,200".to_string()), &None, &None, &None, &None).is_ok());
        
        // 测试网格参数
        assert!(validate_position_parameters(&None, &None, &Some("0".to_string()), &Some("0".to_string()), &None, &None).is_ok());
        
        // 测试冲突参数
        assert!(validate_position_parameters(
            &Some("100,200".to_string()), 
            &Some("100,200".to_string()), 
            &None, &None, &None, &None
        ).is_err());
    }

    #[test]
    fn test_calculate_positions() {
        // 测试单一位置模式
        let single = calculate_positions(3, &Some("100,200".to_string()), "", &None, &None, &None, &None).unwrap();
        assert_eq!(single, vec![(100, 200), (100, 200), (100, 200)]);
        
        // 测试布局模式
        let layout = calculate_positions(2, &None, "100,200,150,250", &None, &None, &None, &None).unwrap();
        assert_eq!(layout, vec![(100, 200), (150, 250)]);
        
        // 测试网格模式
        let grid = calculate_positions(3, &None, "", &Some("0".to_string()), &Some("0".to_string()), &Some("100".to_string()), &Some("50".to_string())).unwrap();
        assert_eq!(grid, vec![(0, 0), (100, 50), (200, 100)]);
    }
}