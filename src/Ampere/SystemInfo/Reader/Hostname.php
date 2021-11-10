<?php

namespace App\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\ValueObject\HostnameValueObject;

class Hostname implements IReader
{
    private const HOSTNAME_PATH = '/proc/sys/kernel/hostname';

    /**
     * Hostname constructor.
     */
    public function __construct()
    {
    }

    /**
     * @throws \Exception
     */
    public function read(): HostnameValueObject
    {
        $fileContents = \file_get_contents(self::HOSTNAME_PATH);
        if (false === $fileContents) {
            throw new \Exception(\sprintf('File %s not found in class: %s', self::HOSTNAME_PATH, __CLASS__));
        }

        $hostname = $this->parse($fileContents);

        /* @phpstan-ignore-next-line */
        return new HostnameValueObject($hostname);
    }

    public function parse(string $fileContents): array|object|string
    {
        return \str_replace(["\r", "\n"], '', $fileContents);
    }
}
