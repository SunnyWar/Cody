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

# Show menu
Write-Host "Select operation:" -ForegroundColor Cyan
Write-Host "  1. Run full orchestration workflow (automatic)"
Write-Host "  2. Analyze only (generate TODO lists)"
Write-Host "  3. Execute next task for each category"
Write-Host "  4. View TODO statistics"
Write-Host "  5. Exit"
Write-Host ""

$choice = Read-Host "Enter choice (1-5)"

switch ($choice) {
    "1" {
        Write-Host "`n🚀 Starting full orchestration workflow..." -ForegroundColor Green
        Write-Host "This will run all three phases: Refactoring → Performance → Features" -ForegroundColor Yellow
        Write-Host ""
        
        $confirm = Read-Host "Continue? (y/n)"
        if ($confirm -eq "y") {
            Set-Location $agentDir
            python orchestrator.py
            Set-Location ..
        }
    }
    
    "2" {
        Write-Host "`n🔍 Running analysis only..." -ForegroundColor Green
        Set-Location $agentDir
        
        Write-Host "`nRefactoring analysis..."
        python refactoring_analyzer.py
        
        Write-Host "`nPerformance analysis..."
        python performance_analyzer.py
        
        Write-Host "`nFeatures analysis..."
        python features_analyzer.py
        
        Set-Location ..
        Write-Host "`n✅ Analysis complete. Check TODO_*.md files." -ForegroundColor Green
    }
    
    "3" {
        Write-Host "`n▶️ Executing next task from each category..." -ForegroundColor Green
        Set-Location $agentDir
        
        Write-Host "`nRefactoring..."
        python refactoring_executor.py next
        
        Write-Host "`nPerformance..."
        python performance_executor.py next
        
        Write-Host "`nFeatures..."
        python features_executor.py next
        
        Set-Location ..
        Write-Host "`n✅ Execution complete." -ForegroundColor Green
    }
    
    "4" {
        Write-Host "`n📊 TODO Statistics:" -ForegroundColor Green
        Set-Location $agentDir
        
        Write-Host "`nRefactoring:"
        python todo_manager.py refactoring
        
        Write-Host "`nPerformance:"
        python todo_manager.py performance
        
        Write-Host "`nFeatures:"
        python todo_manager.py features
        
        Set-Location ..
    }
    
    "5" {
        Write-Host "`n👋 Goodbye!" -ForegroundColor Cyan
        exit 0
    }
    
    default {
        Write-Host "`n❌ Invalid choice. Exiting." -ForegroundColor Red
        exit 1
    }
}

Write-Host "`nDone!" -ForegroundColor Green
