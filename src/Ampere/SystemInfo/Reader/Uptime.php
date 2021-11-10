<?php

namespace App\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\ValueObject\UptimeValueObject;

class Uptime implements IReader
{
    private const UPTIME_PATH = '/proc/uptime';

    public function __construct()
    {
    }

    /**
     * @throws \Exception
     */
    public function read(): UptimeValueObject
    {
        $fileContents = \file_get_contents(self::UPTIME_PATH);
        if (false === $fileContents) {
            throw new \Exception(\sprintf('File %s not found in class: %s', self::UPTIME_PATH, __CLASS__));
        }

        $data = \explode(' ', $fileContents);
        $uptime = (float) $data[0];
        $idleTime = (float) $data[1];

        return new UptimeValueObject($uptime, $idleTime);
    }

    public function parse(string $fileContents): array|object|string
    {
        return '';
    }
}
