<?php

declare(strict_types=1);

namespace App\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;

class UptimeValueObject
{
    public function __construct(private float $uptime, private float $idled)
    {
        $argList = \func_get_args();
        \array_walk($argList, function ($item) {
            if (\is_numeric($item) && $item < 0) {
                $errorMessage = \sprintf('%s cannot accept values less than 0', __CLASS__);
                throw new ValueObjectException($errorMessage);
            }
        });
    }

    public function getUptime(): float
    {
        return $this->uptime;
    }

    public function getIdled(): float
    {
        return $this->idled;
    }
}
