#!/bin/bash

# 默认参数
INPUT_DIR="src"
OUTPUT_FILE="code.txt"
EXTRA_FILES=()
IGNORE_PATTERNS=()

# 解析参数
while [[ $# -gt 0 ]]; do
    case "$1" in
        -i|--input)
            INPUT_DIR="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        -a|--add)
            EXTRA_FILES+=("$2")
            shift 2
            ;;
        --ignore)
            IGNORE_PATTERNS+=("$2")
            shift 2
            ;;
        *)
            echo "未知参数: $1"
            exit 1
            ;;
    esac
done


# 清空或创建输出文件
> "$OUTPUT_FILE"

# 处理额外文件
for file in "${EXTRA_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "\`\`\`${file}" >> "$OUTPUT_FILE"
        cat "$file" >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
        echo "\`\`\`" >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
    else
        echo "警告: 额外文件 $file 不存在" >&2
    fi
done

# ... existing code for processing src directory ...
find "$INPUT_DIR" -type f | while read -r file; do
   # 检查是否匹配忽略模式
    for pattern in "${IGNORE_PATTERNS[@]}"; do
        if [[ "$file" == *"$pattern"* ]]; then
            continue 2
        fi
    done
    
    # 写入文件名标记
    echo "\`\`\`${file}" >> "$OUTPUT_FILE"
    # 写入文件内容
    cat "$file" >> "$OUTPUT_FILE"
    # 写入结束标记
    echo "" >> "$OUTPUT_FILE"
    echo "\`\`\`" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
done

echo "源代码已打包到 $OUTPUT_FILE"