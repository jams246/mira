<?php

namespace App\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;

class CpuUtilizationReadtimeValueObject
{
    public function __construct(private float $totalTime, private float $idleTime)
    {
        $argList = \func_get_args();
        \array_walk($argList, function ($item) {
            if (\is_numeric($item) && $item < 0) {
                $errorMessage = \sprintf('%s cannot accept values less than 0', __CLASS__);
                throw new ValueObjectException($errorMessage);
            }
        });
    }

    public function getTotalTime(): float
    {
        return $this->totalTime;
    }

    public function setTotalTime(float $totalTime): CpuUtilizationReadtimeValueObject
    {
        $this->totalTime = $totalTime;

        return $this;
    }

    public function getIdleTime(): float
    {
        return $this->idleTime;
    }

    public function setIdleTime(float $idleTime): CpuUtilizationReadtimeValueObject
    {
        $this->idleTime = $idleTime;

        return $this;
    }
}
