# Stria Rust - Spec Submodule Update Script
# This script updates the spec submodule to the latest version

param(
    [switch]$AutoCommit = $false,
    [switch]$Help = $false
)

# Show help
if ($Help) {
    Write-Host "Stria Rust - Spec Submodule Update Script" -ForegroundColor Green
    Write-Host ""
    Write-Host "Usage:"
    Write-Host "  .\update-spec.ps1            # Update spec submodule interactively"
    Write-Host "  .\update-spec.ps1 -AutoCommit # Update and auto-commit changes"
    Write-Host "  .\update-spec.ps1 -Help       # Show this help message"
    Write-Host ""
    exit 0
}

# Function to write colored output
function Write-Status {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

Write-Status "Updating spec submodule..." "Cyan"

# Check if we're in the project root
if (-not (Test-Path "Cargo.toml")) {
    Write-Status "Error: This script must be run from the project root directory" "Red"
    exit 1
}

# Check if spec directory exists
if (-not (Test-Path "spec")) {
    Write-Status "Error: spec directory not found" "Red"
    exit 1
}

try {
    # Initialize submodules if needed
    Write-Status "Initializing submodules..." "Yellow"
    git submodule update --init --recursive
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to initialize submodules"
    }

    # Update spec submodule to latest
    Write-Status "Updating spec submodule to latest commit..." "Yellow"
    git submodule update --remote spec
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to update spec submodule"
    }

    # Check if there are any changes
    git diff --quiet spec
    if ($LASTEXITCODE -eq 0) {
        Write-Status "Spec submodule is already up to date" "Green"
    } else {
        Write-Status "Spec submodule has been updated" "Green"
        
        # Show the changes
        Write-Status "Changes in spec submodule:" "Cyan"
        git diff --stat spec
        
        $commitChanges = $AutoCommit
        
        if (-not $AutoCommit) {
            # Ask user if they want to commit the changes
            $response = Read-Host "Do you want to commit these changes? (y/N)"
            $commitChanges = $response -match "^[Yy]$"
        }
        
        if ($commitChanges) {
            # Get the latest commit message from spec submodule
            Push-Location spec
            $latestCommit = git log -1 --pretty=format:"%h - %s"
            Pop-Location
            
            # Commit the submodule update
            git add spec
            $commitMessage = @"
docs: update spec submodule to latest version

Updated to: $latestCommit
"@
            git commit -m $commitMessage
            if ($LASTEXITCODE -eq 0) {
                Write-Status "Changes committed successfully" "Green"
            } else {
                Write-Status "Failed to commit changes" "Red"
                exit 1
            }
        } else {
            Write-Status "Changes not committed. You can commit manually later with:" "Blue"
            Write-Status "   git add spec" "Gray"
            Write-Status "   git commit -m `"docs: update spec submodule to latest version`"" "Gray"
        }
    }

    Write-Status "Spec submodule update complete!" "Green"
} catch {
    Write-Status "Error: $_" "Red"
    exit 1
}
