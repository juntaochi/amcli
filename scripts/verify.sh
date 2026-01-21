#!/bin/bash
# 项目验证脚本 / Project Verification Script

set -e  # 遇到错误立即退出

echo "🦀 AMCLI Project Verification"
echo "=============================="
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 1. Format check
echo "📝 Checking code formatting..."
if cargo fmt -- --check; then
    echo -e "${GREEN}✓${NC} Code is properly formatted"
else
    echo -e "${YELLOW}⚠${NC} Code needs formatting. Run: cargo fmt"
    exit 1
fi
echo ""

# 2. Clippy check
echo "🔍 Running Clippy linter..."
if cargo clippy --all-features -- -D warnings; then
    echo -e "${GREEN}✓${NC} Clippy checks passed"
else
    echo -e "${RED}✗${NC} Clippy found issues"
    exit 1
fi
echo ""

# 3. Tests
echo "🧪 Running tests..."
if cargo test --all-features; then
    echo -e "${GREEN}✓${NC} All tests passed"
else
    echo -e "${RED}✗${NC} Some tests failed"
    exit 1
fi
echo ""

# 4. Build
echo "🏗️ Building project..."
if cargo build --all-features; then
    echo -e "${GREEN}✓${NC} Build successful"
else
    echo -e "${RED}✗${NC} Build failed"
    exit 1
fi
echo ""

# 5. Documentation
echo "📚 Checking documentation..."
if cargo doc --no-deps --all-features; then
    echo -e "${GREEN}✓${NC} Documentation generated"
else
    echo -e "${YELLOW}⚠${NC} Documentation has issues"
fi
echo ""

echo "=============================="
echo -e "${GREEN}✅ All checks passed!${NC}"
echo ""
echo "You're ready to:"
echo "1. Commit your changes"
echo "2. Push to remote"
echo "3. Create a pull request"
