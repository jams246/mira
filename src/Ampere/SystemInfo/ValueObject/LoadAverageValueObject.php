<?php

declare(strict_types=1);

namespace App\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;

class LoadAverageValueObject
{
    public function __construct(private float $minute, private float $fiveMinute, private float $fifteenMinute)
    {
        $argList = \func_get_args();
        \array_walk($argList, function ($item) {
            if (\is_numeric($item) && $item < 0) {
                $errorMessage = \sprintf('%s cannot accept values less than 0', __CLASS__);
                throw new ValueObjectException($errorMessage);
            }
        });
    }

    public function getMinute(): float
    {
        return $this->minute;
    }

    public function getFiveMinute(): float
    {
        return $this->fiveMinute;
    }

    public function getFifteenMinute(): float
    {
        return $this->fifteenMinute;
    }
}
