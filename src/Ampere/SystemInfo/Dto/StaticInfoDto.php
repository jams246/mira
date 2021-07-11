<?php

namespace App\Ampere\SystemInfo\Dto;

use App\Ampere\SystemInfo\ValueObject\BitVersionValueObject;
use App\Ampere\SystemInfo\ValueObject\CpuCoreCountValueObject;
use App\Ampere\SystemInfo\ValueObject\CpuModelNameValueObject;
use App\Ampere\SystemInfo\ValueObject\HostnameValueObject;
use App\Ampere\SystemInfo\ValueObject\IPAddressValueObject;
use App\Ampere\SystemInfo\ValueObject\KernelValueObject;

class StaticInfoDto
{
    private BitVersionValueObject $bitVersion;
    private CpuCoreCountValueObject $coreCount;
    private CpuModelNameValueObject $cpuModelName;
    private IPAddressValueObject $hostIPAddress;
    private KernelValueObject $kernel;
    private IPAddressValueObject $publicIPAddress;
    private HostnameValueObject $hostname;

    public function getBitVersion(): BitVersionValueObject
    {
        return $this->bitVersion;
    }

    public function setBitVersion(BitVersionValueObject $bitVersion): StaticInfoDto
    {
        $this->bitVersion = $bitVersion;

        return $this;
    }

    public function getCoreCount(): CpuCoreCountValueObject
    {
        return $this->coreCount;
    }

    public function setCoreCount(CpuCoreCountValueObject $coreCount): StaticInfoDto
    {
        $this->coreCount = $coreCount;

        return $this;
    }

    public function getCpuModelName(): CpuModelNameValueObject
    {
        return $this->cpuModelName;
    }

    public function setCpuModelName(CpuModelNameValueObject $cpuModelName): StaticInfoDto
    {
        $this->cpuModelName = $cpuModelName;

        return $this;
    }

    public function getHostIPAddress(): IPAddressValueObject
    {
        return $this->hostIPAddress;
    }

    public function setHostIPAddress(IPAddressValueObject $hostIPAddress): StaticInfoDto
    {
        $this->hostIPAddress = $hostIPAddress;

        return $this;
    }

    public function getKernel(): KernelValueObject
    {
        return $this->kernel;
    }

    public function setKernel(KernelValueObject $kernel): StaticInfoDto
    {
        $this->kernel = $kernel;

        return $this;
    }

    public function getPublicIPAddress(): IPAddressValueObject
    {
        return $this->publicIPAddress;
    }

    public function setPublicIPAddress(IPAddressValueObject $publicIPAddress): StaticInfoDto
    {
        $this->publicIPAddress = $publicIPAddress;

        return $this;
    }

    public function getHostname(): HostnameValueObject
    {
        return $this->hostname;
    }

    public function setHostname(HostnameValueObject $hostname): StaticInfoDto
    {
        $this->hostname = $hostname;

        return $this;
    }
}
