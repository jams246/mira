<?php

namespace App\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;

class CpuUtilizationValueObject
{
    public function __construct(private float $utilization, private float $timeRecorded)
    {
        $argList = \func_get_args();
        \array_walk($argList, function ($item) {
            if (\is_numeric($item) && $item < 0) {
                $errorMessage = \sprintf('%s cannot accept values less than 0', __CLASS__);
                throw new ValueObjectException($errorMessage);
            }
        });
    }

    public function getUtilization(): float
    {
        return $this->utilization;
    }

    public function getTimeRecorded(): float
    {
        return $this->timeRecorded;
    }
}
