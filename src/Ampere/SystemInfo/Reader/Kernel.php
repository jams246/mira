<?php

namespace App\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\ValueObject\KernelValueObject;

class Kernel implements IReader
{
    private const KERNEL_PATH = '/proc/version';

    /**
     * Kernel constructor.
     */
    public function __construct()
    {
    }

    /**
     * @throws \Exception
     */
    public function read(): KernelValueObject
    {
        $fileContents = \file_get_contents(self::KERNEL_PATH);
        if (false === $fileContents) {
            throw new \Exception(\sprintf('File %s not found in class: %s', self::KERNEL_PATH, __CLASS__));
        }

        $response = (array) $this->parse($fileContents);
        $position = $response[0][2][1];

        $kernel = \substr($fileContents, 0, $position);

        return new KernelValueObject($kernel);
    }

    public function parse(string $fileContents): array|object|string
    {
        \preg_match_all('/ /', $fileContents, $matches, PREG_OFFSET_CAPTURE);

        return $matches;
    }
}
