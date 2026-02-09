# Run the Cody AI Orchestrator
# This PowerShell script starts the multi-phase improvement workflow

Write-Host "=" -ForegroundColor Cyan -NoNewline; Write-Host ("=" * 69) -ForegroundColor Cyan
Write-Host "Cody AI Orchestration System" -ForegroundColor Green
Write-Host "=" -ForegroundColor Cyan -NoNewline; Write-Host ("=" * 69) -ForegroundColor Cyan
Write-Host ""

# Check if Python is available
if (-not (Get-Command python -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Python not found. Please install Python 3.8 or later." -ForegroundColor Red
    exit 1
}

# Check if we're in the right directory
$agentDir = Join-Path $PSScriptRoot "cody-agent"
if (-not (Test-Path $agentDir)) {
    Write-Host "❌ cody-agent directory not found." -ForegroundColor Red
    exit 1
}

# Check if required Python packages are installed
Write-Host "Checking dependencies..." -ForegroundColor Yellow
$packages = @("openai", "requests")

foreach ($pkg in $packages) {
    $installed = python -c "import $pkg" 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Package '$pkg' not found. Installing..." -ForegroundColor Yellow
        python -m pip install $pkg
    }
}

Write-Host "✅ Dependencies OK" -ForegroundColor Green
Write-Host ""

# Helper to select a category
function Select-Category {
    Write-Host "Select category:" -ForegroundColor Cyan
    Write-Host "  1. Refactoring"
    Write-Host "  2. Performance"
    Write-Host "  3. Features"
    Write-Host "  4. Clippy"
    Write-Host ""

    $categoryChoice = Read-Host "Enter choice (1-4)"
    switch ($categoryChoice) {
        "1" { return "refactoring" }
        "2" { return "performance" }
        "3" { return "features" }
        "4" { return "clippy" }
        default {
            Write-Host "`n❌ Invalid category choice." -ForegroundColor Red
            return $null
        }
    }
}

# Show menu
Write-Host "Select operation:" -ForegroundColor Cyan
Write-Host "  1. Run single improvement (orchestrator)"
Write-Host "  2. Analyze all categories (refactoring, performance, features, clippy)"
Write-Host "  3. Analyze a single category"
Write-Host "  4. Execute next task from a single category"
Write-Host "  5. Execute next task from all categories"
Write-Host "  6. View TODO statistics"
Write-Host "  7. Exit"
Write-Host ""

$choice = Read-Host "Enter choice (1-7)"

switch ($choice) {
    "1" {
        Write-Host "`n🚀 Starting single improvement run..." -ForegroundColor Green
        Write-Host "This runs exactly one orchestrated task and exits." -ForegroundColor Yellow
        Write-Host ""

        $confirm = Read-Host "Continue? (y/n)"
        if ($confirm -eq "y") {
            Set-Location $agentDir
            python orchestrator.py
            Set-Location ..
        }
    }

    "2" {
        Write-Host "`n🔍 Running analysis for all categories..." -ForegroundColor Green
        Set-Location $agentDir

        Write-Host "`nRefactoring analysis..."
        python refactoring_analyzer.py

        Write-Host "`nPerformance analysis..."
        python performance_analyzer.py

        Write-Host "`nFeatures analysis..."
        python features_analyzer.py

        Write-Host "`nClippy analysis..."
        python clippy_analyzer.py

        Set-Location ..
        Write-Host "`n✅ Analysis complete. Check TODO_*.md files." -ForegroundColor Green
    }

    "3" {
        $category = Select-Category
        if ($null -ne $category) {
            Write-Host "`n🔍 Running analysis for $category..." -ForegroundColor Green
            Set-Location $agentDir
            switch ($category) {
                "refactoring" { python refactoring_analyzer.py }
                "performance" { python performance_analyzer.py }
                "features" { python features_analyzer.py }
                "clippy" { python clippy_analyzer.py }
            }
            Set-Location ..
            Write-Host "`n✅ Analysis complete. Check TODO_*.md files." -ForegroundColor Green
        }
    }

    "4" {
        $category = Select-Category
        if ($null -ne $category) {
            Write-Host "`n▶️ Executing next task for $category..." -ForegroundColor Green
            Set-Location $agentDir
            switch ($category) {
                "refactoring" { python refactoring_executor.py next }
                "performance" { python performance_executor.py next }
                "features" { python features_executor.py next }
                "clippy" { python clippy_executor.py next }
            }
            Set-Location ..
            Write-Host "`n✅ Execution complete." -ForegroundColor Green
        }
    }

    "5" {
        Write-Host "`n▶️ Executing next task from all categories..." -ForegroundColor Green
        Set-Location $agentDir

        Write-Host "`nRefactoring..."
        python refactoring_executor.py next

        Write-Host "`nPerformance..."
        python performance_executor.py next

        Write-Host "`nFeatures..."
        python features_executor.py next

        Write-Host "`nClippy..."
        python clippy_executor.py next

        Set-Location ..
        Write-Host "`n✅ Execution complete." -ForegroundColor Green
    }

    "6" {
        Write-Host "`n📊 TODO Statistics:" -ForegroundColor Green
        Set-Location $agentDir

        Write-Host "`nRefactoring:"
        python todo_manager.py refactoring

        Write-Host "`nPerformance:"
        python todo_manager.py performance

        Write-Host "`nFeatures:"
        python todo_manager.py features

        Write-Host "`nClippy:"
        python todo_manager.py clippy

        Set-Location ..
    }

    "7" {
        Write-Host "`n👋 Goodbye!" -ForegroundColor Cyan
        exit 0
    }
    
    default {
        Write-Host "`n❌ Invalid choice. Exiting." -ForegroundColor Red
        exit 1
    }
}

Write-Host "`nDone!" -ForegroundColor Green
