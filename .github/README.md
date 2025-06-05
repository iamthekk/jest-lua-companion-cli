# GitHub Actions Workflows

## publish-npm.yml

This workflow automatically publishes the jest-lua-companion-cli package to npm when:

1. A new version tag (starting with 'v') is pushed to the repository
2. The workflow is manually triggered via GitHub Actions UI

### Prerequisites

To use this workflow, you need to:

1. Set up an NPM_TOKEN secret in your GitHub repository settings
   - Go to your repository Settings > Secrets and Variables > Actions
   - Add a new repository secret named `NPM_TOKEN`
   - The value should be an npm access token with publish permissions

### How it works

1. **Triggers**: Runs on version tags (v*) or manual dispatch
2. **Environment**: Runs on Windows (windows-latest) since the package is Windows-only
3. **Build Process**:
   - Sets up Rust toolchain for Windows
   - Caches cargo dependencies for faster builds
   - Builds the Rust binary in release mode
   - Verifies the Windows binary (.exe) exists
4. **Publishing**:
   - Sets up Node.js
   - Runs `npm publish` which automatically triggers the prepare script
   - Uses the NPM_TOKEN secret for authentication

### Usage

To publish a new version:

1. Update the version in `package.json`
2. Commit the changes
3. Create and push a version tag:
   ```bash
   git tag v0.3.1
   git push origin v0.3.1
   ```
4. The workflow will automatically build and publish to npm