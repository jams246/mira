<?php

namespace App\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;

class SwapMemoryValueObject
{
    public function __construct(private int $total, private int $free, private int $cached)
    {
        $argList = \func_get_args();
        \array_walk($argList, function ($item) {
            if (\is_numeric($item) && $item < 0) {
                $errorMessage = \sprintf('%s cannot accept values less than 0', __CLASS__);
                throw new ValueObjectException($errorMessage);
            }
        });
    }

    public function getTotal(): int
    {
        return $this->total;
    }

    public function getFree(): int
    {
        return $this->free;
    }

    public function getCached(): int
    {
        return $this->cached;
    }
}
