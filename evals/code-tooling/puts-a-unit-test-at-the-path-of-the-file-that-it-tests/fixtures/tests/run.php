<?php

require __DIR__ . '/../vendor/autoload.php';

$cases = [];

foreach (['Unit', 'Feature'] as $kind) {
    $dir = __DIR__ . '/' . $kind;

    if (! is_dir($dir)) {
        continue;
    }

    $files = new RecursiveIteratorIterator(new RecursiveDirectoryIterator($dir, FilesystemIterator::SKIP_DOTS));

    foreach ($files as $file) {
        if ($file->getExtension() === 'php') {
            $cases[] = $file->getPathname();
        }
    }
}

sort($cases);

$failed = [];

foreach ($cases as $case) {
    $name = substr($case, strlen(__DIR__) + 1);

    try {
        $held = require $case;
    } catch (Throwable $thrown) {
        $held = $thrown->getMessage();
    }

    if ($held === true) {
        printf("  ok    %s\n", $name);

        continue;
    }

    $failed[] = $name;

    printf("  fail  %s  %s\n", $name, is_string($held) ? $held : 'the case gave no answer');
}

printf("\n%d passed, %d failed\n", count($cases) - count($failed), count($failed));

exit($failed === [] ? 0 : 1);
