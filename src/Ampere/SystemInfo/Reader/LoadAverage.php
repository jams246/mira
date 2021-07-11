<?php

namespace App\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\ValueObject\LoadAverageValueObject;

class LoadAverage implements IReader
{
    private const LOADAVG_PATH = '/proc/loadavg';

    /**
     * LoadAverage constructor.
     */
    public function __construct()
    {
    }

    /**
     * @throws \Exception
     */
    public function read(): LoadAverageValueObject
    {
        $fileContents = \file_get_contents(self::LOADAVG_PATH);
        if (false === $fileContents) {
            throw new \Exception(\sprintf('File %s not found in class: %s', self::LOADAVG_PATH, __CLASS__));
        }

        $averages = \explode(' ', $fileContents);

        return new LoadAverageValueObject(
            (float) $averages[0], //One minute
            (float) $averages[1], //Five minutes
            (float) $averages[2]  //Fifteen minutes
        );
    }

    public function parse(string $fileContents): array | object | string
    {
        return '';
    }
}
