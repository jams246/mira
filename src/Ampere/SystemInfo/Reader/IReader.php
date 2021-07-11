<?php

namespace App\Ampere\SystemInfo\Reader;

interface IReader
{
    /* @phpstan-ignore-next-line */
    public function read();

    public function parse(string $fileContents): array | object | string;
}
