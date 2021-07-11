<?php

declare(strict_types=1);

namespace App\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;

class MemoryValueObject
{
    public function __construct(private int $total, private int $free, private int $available, private int $buffers, private int $cached, private float $percentageUsed, private float $timeRecorded)
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

    public function getAvailable(): int
    {
        return $this->available;
    }

    public function getBuffers(): int
    {
        return $this->buffers;
    }

    public function getCached(): int
    {
        return $this->cached;
    }

    public function getPercentageUsed(): float
    {
        return $this->percentageUsed;
    }

    public function getTimeRecorded(): float
    {
        return $this->timeRecorded;
    }
}
