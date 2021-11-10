<?php

namespace App\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\ValueObject\IPAddressValueObject;
use Symfony\Component\Process\Process;

class PublicIPAddress implements IReader
{
    private string $ipAddress = 'Unknown';

    private const IP_ADDRESS_WEBSITES = [
        'checkip.amazonaws.com',
        'ifconfig.me',
        'icanhazip.com',
        'ipecho.net/plain',
        'ifconfig.co',
    ];

    public function __construct()
    {
    }

    public function read(): IPAddressValueObject
    {
        $classMethods = \array_filter(
            \get_class_methods(self::class),
            function ($var) {
                return \str_contains($var, 'try');
            }
        );

        foreach ($classMethods as $method) {
            if (\filter_var($this->ipAddress, FILTER_VALIDATE_IP)) {
                break;
            }
            /* @phpstan-ignore-next-line */
            \call_user_func([$this, $method]);
        }

        $ipAddress = \str_replace(["\r", "\n"], '', $this->ipAddress);

        return new IPAddressValueObject($ipAddress);
    }

    private function tryDig(): void
    {
        $process = new Process(['dig', '+short', 'myip.opendns.com', '@resolver1.opendns.com']);
        $process->run();
        if ($process->isSuccessful()) {
            $this->ipAddress = $process->getOutput();
        }
    }

    private function tryCurl(): void
    {
        foreach (self::IP_ADDRESS_WEBSITES as $address) {
            $process = new Process(['curl', $address]);
            $process->run();
            if ($process->isSuccessful()) {
                $this->ipAddress = $process->getOutput();
                break;
            }
        }
    }

    public function parse(string $fileContents): array|object|string
    {
        return '';
    }
}
