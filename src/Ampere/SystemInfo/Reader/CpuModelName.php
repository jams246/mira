<?php

declare(strict_types=1);

namespace App\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\ValueObject\CpuModelNameValueObject;

class CpuModelName implements IReader
{
    private const CPUINFO_PATH = '/proc/cpuinfo';

    /**
     * CpuModelName constructor.
     */
    public function __construct()
    {
    }

    /**
     * @throws \Exception
     */
    public function read(): CpuModelNameValueObject
    {
        $fileContents = \file_get_contents(self::CPUINFO_PATH);
        if (false === $fileContents) {
            throw new \Exception(\sprintf('File %s not found in class: %s', self::CPUINFO_PATH, __CLASS__));
        }
        $readings = $this->parse($fileContents);

        /* @phpstan-ignore-next-line */
        return new CpuModelNameValueObject($readings->{'model name'});
    }

    public function parse(string $fileContents): object
    {
        \preg_match_all('/^([^:]+)\:\s+(.*)/m', $fileContents, $matches);

        \array_walk($matches[1], function (&$item) {
            $item = \str_replace("\t", '', $item);
        });

        return (object) \array_combine($matches[1], $matches[2]);
    }
}
