<?php

namespace App\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\ValueObject\SwapMemoryValueObject;

class SwapMemory implements IReader
{
    private const MEMORY_PATH = '/proc/meminfo';

    public function __construct()
    {
    }

    /**
     * @throws \Exception
     */
    public function read(): SwapMemoryValueObject
    {
        $fileContents = \file_get_contents(self::MEMORY_PATH);
        if (false === $fileContents) {
            throw new \Exception(\sprintf('File %s not found in class: %s', self::MEMORY_PATH, __CLASS__));
        }

        \preg_match_all('/^([^:]+)\:\s+(\d+)/m', $fileContents, $matches);

        $memoryInfo = (object) \array_combine($matches[1], $matches[2]);
        unset($matches);

        $swapTotal = $memoryInfo->SwapTotal;
        $swapFree = $memoryInfo->SwapFree;
        $swapCached = $memoryInfo->SwapCached;

        //Insert the values into the value object and convert the kB to bytes
        return new SwapMemoryValueObject(
            $swapTotal,
            $swapFree,
            $swapCached
        );
    }

    public function parse(string $fileContents): array | object | string
    {
        return '';
    }
}
