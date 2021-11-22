<?php

namespace App\Ampere\SystemInfo\ValueObject;

class DiskDetailsValueObject
{
    public function __construct(
        private string $device,
        private int $total,
        private int $used,
        private int $available,
        private string $percentageUsed,
        private string $mountedAt
    ) {
    }

    public function getDevice(): string
    {
        return $this->device;
    }

    public function getTotal(): int
    {
        return $this->total;
    }

    public function getUsed(): int
    {
        return $this->used;
    }

    public function getAvailable(): int
    {
        return $this->available;
    }

    public function getPercentageUsed(): string
    {
        return $this->percentageUsed;
    }

    public function getMountedAt(): string
    {
        return $this->mountedAt;
    }
}
