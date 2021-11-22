<?php

namespace App\Ampere\SystemInfo\Dto;

use App\Ampere\SystemInfo\ValueObject\CpuUtilizationValueObject;
use App\Ampere\SystemInfo\ValueObject\LoadAverageValueObject;

class LiveInfoDto
{
    private ?CpuUtilizationValueObject $cpuUtilization = null;
    private ?LoadAverageValueObject $loadAverage = null;
    private ?object $memory = null;
    private ?\ArrayObject $processList = null;
    private ?object $swapMemory = null;
    private ?object $uptime = null;
    private ?\ArrayObject $dockerList = null;
    private ?\ArrayObject $diskList = null;

    public function getCpuUtilization(): ?CpuUtilizationValueObject
    {
        return $this->cpuUtilization;
    }

    public function setCpuUtilization(?CpuUtilizationValueObject $cpuUtilization): LiveInfoDto
    {
        $this->cpuUtilization = $cpuUtilization;

        return $this;
    }

    public function getLoadAverage(): ?LoadAverageValueObject
    {
        return $this->loadAverage;
    }

    public function setLoadAverage(?LoadAverageValueObject $loadAverage): LiveInfoDto
    {
        $this->loadAverage = $loadAverage;

        return $this;
    }

    public function getMemory(): ?object
    {
        return $this->memory;
    }

    public function setMemory(?object $memory): LiveInfoDto
    {
        $this->memory = $memory;

        return $this;
    }

    public function getProcessList(): ?\ArrayObject
    {
        return $this->processList;
    }

    public function setProcessList(?\ArrayObject $processList): LiveInfoDto
    {
        $this->processList = $processList;

        return $this;
    }

    public function getSwapMemory(): ?object
    {
        return $this->swapMemory;
    }

    public function setSwapMemory(?object $swapMemory): LiveInfoDto
    {
        $this->swapMemory = $swapMemory;

        return $this;
    }

    public function getUptime(): ?object
    {
        return $this->uptime;
    }

    public function setUptime(?object $uptime): LiveInfoDto
    {
        $this->uptime = $uptime;

        return $this;
    }

    public function getDockerList(): ?\ArrayObject
    {
        return $this->dockerList;
    }

    public function setDockerList(?\ArrayObject $dockerList): LiveInfoDto
    {
        $this->dockerList = $dockerList;

        return $this;
    }

    public function getDiskList(): ?\ArrayObject
    {
        return $this->diskList;
    }

    public function setDiskList(?\ArrayObject $diskList): LiveInfoDto
    {
        $this->diskList = $diskList;

        return $this;
    }
}
