<?php

namespace App\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;

class ProcessInfoValueObject
{
    private const ALLOWED_LIST = ['R (running)', 'S (sleeping)', 'I (idle)'];

    public function __construct(
        private int $pid,
        private string $processName,
        private int $memoryUsage,
        private string $state,
        private float $cpuUsage,
        private float $upTime
    ) {
        $argList = \func_get_args();
        \array_walk($argList, function ($item) {
            if (\is_numeric($item) && $item < 0) {
                $errorMessage = \sprintf('%s cannot accept values less than 0', __CLASS__);
                throw new ValueObjectException($errorMessage);
            }
        });

        if (!\in_array($this->state, self::ALLOWED_LIST)) {
            $errorMessage = \sprintf(
                '%s only accepts allowed values. Input was %s',
                __CLASS__,
                $state
            );
            throw new ValueObjectException($errorMessage);
        }
    }

    public function getPid(): int
    {
        return $this->pid;
    }

    public function getProcessName(): string
    {
        return $this->processName;
    }

    public function getMemoryUsage(): int
    {
        return $this->memoryUsage;
    }

    public function getState(): string
    {
        return $this->state;
    }

    public function setState(string $state): ProcessInfoValueObject
    {
        $this->state = $state;

        return $this;
    }

    public function getCpuUsage(): float
    {
        return $this->cpuUsage;
    }

    public function getUpTime(): float
    {
        return $this->upTime;
    }
}
