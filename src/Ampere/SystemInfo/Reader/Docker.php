<?php

declare(strict_types=1);

namespace App\Ampere\SystemInfo\Reader;

use Symfony\Component\Cache\Adapter\AdapterInterface;

class Docker implements IReader
{
    public function __construct(private AdapterInterface $cache)
    {
    }

    public function read(): ?array
    {
        $dockerList = $this->cache->getItem('docker.list');

        $response = null;
        if ($dockerList->isHit()) {
            $response = $dockerList->get();
        }

        return $response;
    }

    public function parse(string $fileContents): array | object | string
    {
        return '';
    }
}
