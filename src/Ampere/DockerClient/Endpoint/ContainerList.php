<?php

namespace App\Ampere\DockerClient\Endpoint;

use App\Ampere\DockerClient\ValueObject\MethodValueObject;
use App\Ampere\DockerClient\ValueObject\UriValueObject;

class ContainerList implements IEndpointInterface
{
    private const METHOD = 'GET';
    private const URI = '/containers/json';

    private MethodValueObject $method;
    private UriValueObject $uri;
    private bool $streamBlocked = true;

    public function __construct()
    {
        $this->method = new MethodValueObject(self::METHOD);
        $this->uri = new UriValueObject(self::URI);
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
