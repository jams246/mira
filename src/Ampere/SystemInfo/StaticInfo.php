<?php

declare(strict_types=1);

namespace App\Ampere\SystemInfo;

use App\Ampere\SystemInfo\Dto\StaticInfoDto;
use App\Ampere\SystemInfo\Reader\BitVersion;
use App\Ampere\SystemInfo\Reader\CpuCoreCount;
use App\Ampere\SystemInfo\Reader\CpuModelName;
use App\Ampere\SystemInfo\Reader\HostIPAddress;
use App\Ampere\SystemInfo\Reader\Hostname;
use App\Ampere\SystemInfo\Reader\Kernel;
use App\Ampere\SystemInfo\Reader\PublicIPAddress;

class StaticInfo
{
    public function __construct(
        private BitVersion $bitVersion,
        private CpuCoreCount $cpuCoreCount,
        private CpuModelName $cpuModelName,
        private HostIPAddress $hostIPAddress,
        private Hostname $hostname,
        private PublicIPAddress $publicIPAddress,
        private Kernel $kernel
    ) {
    }

    public function getDto(): StaticInfoDto
    {
        return (new StaticInfoDto())
            ->setBitVersion($this->bitVersion->read())
            ->setCoreCount($this->cpuCoreCount->read())
            ->setCpuModelName($this->cpuModelName->read())
            ->setHostIPAddress($this->hostIPAddress->read())
            ->setHostname($this->hostname->read())
            ->setPublicIPAddress($this->publicIPAddress->read())
            ->setKernel($this->kernel->read());
    }
}
