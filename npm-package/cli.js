#!/usr/bin/env node
'use strict';

const path = require('path');
const { spawnSync } = require('child_process');

// Resolve the native Xerv binary bundled in this npm package.
function getBinaryPath() {
    const platform = process.platform;
    const arch = process.arch;

    if (platform !== 'linux' || arch !== 'x64') {
        console.error('xerv: Unsupported platform. Currently only Linux x86_64 is supported.');
        process.exit(1);
    }

    const binDir = path.join(__dirname, 'bin');
    const binaryName = 'xerv';
    const binaryPath = path.join(binDir, binaryName);

    // Check that the binary exists.
    try {
        require('fs').accessSync(binaryPath, require('fs').constants.X_OK);
    } catch (e) {
        console.error('xerv: Native binary not found at ' + binaryPath);
        console.error('xerv: This may indicate a corrupted installation.');
        process.exit(1);
    }

    return binaryPath;
}

const binaryPath = getBinaryPath();

// Spawn the native binary with all original arguments — zero logic duplication.
const result = spawnSync(binaryPath, process.argv.slice(2), {
    stdio: 'inherit',
    env: process.env,
});

// Propagate exit code from the native binary.
if (result.error) {
    console.error('xerv: Failed to execute native binary:', result.error.message);
    process.exit(1);
}

process.exit(result.status !== null ? result.status : 1);
