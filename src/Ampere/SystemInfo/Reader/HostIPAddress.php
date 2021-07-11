<?php

namespace App\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\ValueObject\IPAddressValueObject;
use Symfony\Component\Process\Process;

class HostIPAddress implements IReader
{
    /**
     * HostIPAddress constructor.
     */
    public function __construct()
    {
    }

    public function read(): IPAddressValueObject
    {
        $ipAddress = 'Unknown';
        $process = new Process(['hostname', '-i']);
        $process->run();
        if ($process->isSuccessful()) {
            $ipAddress = $this->parse($process->getOutput());
        }

        return new IPAddressValueObject($ipAddress);
    }

    public function parse(string $fileContents): string
    {
        return \str_replace(["\r", "\n"], '', $fileContents);
    }
}
