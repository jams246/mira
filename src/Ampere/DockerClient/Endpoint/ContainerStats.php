<?php

namespace App\Ampere\DockerClient\Endpoint;

use App\Ampere\DockerClient\ValueObject\ContainerIdValueObject;
use App\Ampere\DockerClient\ValueObject\MethodValueObject;
use App\Ampere\DockerClient\ValueObject\UriValueObject;

class ContainerStats implements IEndpointInterface
{
    private const METHOD = 'GET';
    private const URI = '/containers/|CONTAINERID|/stats?stream=true&one-shot=false';

    private MethodValueObject $method;
    private UriValueObject $uri;
    private bool $streamBlocked = false;

    public function __construct(ContainerIdValueObject $containerId)
    {
        $this->method = new MethodValueObject(self::METHOD);

        $uri = \str_replace('|CONTAINERID|', $containerId->getValue(), self::URI);
        $this->uri = new UriValueObject($uri);
    }

    public function getMethod(): MethodValueObject
    {
        return $this->method;
    }

    public function getUri(): UriValueObject
    {
        return $this->uri;
    }

    public function isStreamBlocked(): bool
    {
        return $this->streamBlocked;
    }
}
