<?php

namespace App\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\ValueObject\BitVersionValueObject;

class BitVersion implements IReader
{
    private const CPUINFO_PATH = '/proc/cpuinfo';

    public function __construct()
    {
    }

    /**
     * @throws \Exception
     */
    public function read(): BitVersionValueObject
    {
        $fileContents = \file_get_contents(self::CPUINFO_PATH);
        if (false === $fileContents) {
            throw new \Exception(\sprintf('File %s not found in class: %s', self::CPUINFO_PATH, __CLASS__));
        }
        $readings = $this->parse($fileContents);

        $bitVersion = '32bit';
        /*
         * The lm flag denotes long mode CPU which is 64 bit.
         */
        /* @phpstan-ignore-next-line */
        if (\strpos($readings->flags, ' lm ') > 0) {
            $bitVersion = '64bit';
        }

        return new BitVersionValueObject($bitVersion);
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
