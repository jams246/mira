<?php

namespace App\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\ValueObject\MemoryValueObject;

class Memory implements IReader
{
    private const MEMORY_PATH = '/proc/meminfo';

    /**
     * Memory constructor.
     */
    public function __construct()
    {
    }

    /**
     * @throws \Exception
     */
    public function read(): MemoryValueObject
    {
        $fileContents = \file_get_contents(self::MEMORY_PATH);

        if (false === $fileContents) {
            throw new \Exception(\sprintf('File %s not found in class: %s', self::MEMORY_PATH, __CLASS__));
        }

        $memoryInfo = (object) $this->parse($fileContents);

        $percentageUsed = $this->calculateMemoryUsage($memoryInfo);

        $timeRecorded = \microtime(true) * 1000;

        return new MemoryValueObject(
            $memoryInfo->MemTotal,
            $memoryInfo->MemFree,
            $memoryInfo->MemAvailable,
            $memoryInfo->Buffers,
            $memoryInfo->Cached,
            $percentageUsed,
            $timeRecorded
        );
    }

    public function parse(string $fileContents): array|object|string
    {
        \preg_match_all('/^([^:]+)\:\s+(\d+)/m', $fileContents, $matches);

        return (object) \array_combine($matches[1], $matches[2]);
    }

    public function calculateMemoryUsage(object $memoryInfo): float
    {
        return \ceil((($memoryInfo->MemTotal - $memoryInfo->MemFree - $memoryInfo->Buffers - $memoryInfo->Cached) / $memoryInfo->MemTotal) * 100);
    }
}
