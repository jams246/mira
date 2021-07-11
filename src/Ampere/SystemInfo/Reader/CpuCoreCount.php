<?php

declare(strict_types=1);

namespace App\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\ValueObject\CpuCoreCountValueObject;

class CpuCoreCount implements IReader
{
    private const CPUINFO_PATH = '/proc/cpuinfo';

    public function __construct()
    {
    }

    /**
     * @throws \Exception
     */
    public function read(): CpuCoreCountValueObject
    {
        $fileContents = \file_get_contents(self::CPUINFO_PATH);
        if (false === $fileContents) {
            throw new \Exception(\sprintf('File %s not found in class: %s', self::CPUINFO_PATH, __CLASS__));
        }
        $matches = $this->parse($fileContents);

        $cpuCoreCount = \array_count_values($matches[1])['processor'];

        return new CpuCoreCountValueObject($cpuCoreCount);
    }

    public function parse(string $fileContents): array
    {
        \preg_match_all('/^([^:]+)\:/m', $fileContents, $matches);

        \array_walk($matches[1], function (&$item) {
            $item = \str_replace(["\t", "\n"], '', $item);
        });

        return $matches;
    }
}
