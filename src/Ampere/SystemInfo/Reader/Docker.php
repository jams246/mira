<?php

declare(strict_types=1);

namespace App\Ampere\SystemInfo\Reader;

use Psr\Cache\CacheItemPoolInterface;

class Docker implements IReader
{
    public function __construct(private CacheItemPoolInterface $cache)
    {
    }

    public function read(): ?array
    {
        $dockerList = $this->cache->getItem('docker.list');

        $response = null;
        if ($dockerList->isHit()) {
            $response = $dockerList->get();
        }

        /* @phpstan-ignore-next-line */
        return $response;
    }

    public function parse(string $fileContents): array|object|string
    {
        return '';
    }
}
