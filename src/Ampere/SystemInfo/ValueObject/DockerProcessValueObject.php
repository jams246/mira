<?php

namespace App\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;

class DockerProcessValueObject
{
    public function __construct(
        private string $name,
        private string $state,
        private float $cpu,
        private int $memory,
        private float $upTime
    ) {
        $argList = \func_get_args();
        \array_walk($argList, function ($item) {
            if (\is_numeric($item) && $item < 0) {
                $errorMessage = \sprintf('%s cannot accept values less than 0', __CLASS__);
                throw new ValueObjectException($errorMessage);
            }
        });
    }

    public function getName(): string
    {
        return $this->name;
    }

    public function getState(): string
    {
        return $this->state;
    }

    public function getCpu(): float
    {
        return $this->cpu;
    }

    public function getMemory(): int
    {
        return $this->memory;
    }

    public function getUpTime(): float
    {
        return $this->upTime;
    }
}
