<?php

namespace App\Ampere\DockerClient\Request;

use App\Ampere\DockerClient\Endpoint\IEndpointInterface;
use App\Ampere\DockerClient\ValueObject\MethodValueObject;
use App\Ampere\DockerClient\ValueObject\UriValueObject;

class Context
{
    private string $context = "|METHOD| |URI| HTTP/1.0\r\nHost: unixsocket\r\nAccept: application/json\r\n\r\n";

    private function __construct(MethodValueObject $method, UriValueObject $uri, private bool $isStreamBlocked)
    {
        $this->setMethod($method);
        $this->setUri($uri);
    }

    public static function create(IEndpointInterface $endpoint)
    {
        return new Context(
            $endpoint->getMethod(),
            $endpoint->getUri(),
            $endpoint->isStreamBlocked()
        );
    }

    public function getContext(): string
    {
        return $this->context;
    }

    public function isStreamBlocked(): bool
    {
        return $this->isStreamBlocked;
    }

    private function setMethod(MethodValueObject $method): Context
    {
        $this->context = \str_replace(
            '|METHOD|',
            $method->getValue(),
            $this->context
        );

        return $this;
    }

    private function setUri(UriValueObject $uri): Context
    {
        $this->context = \str_replace(
            '|URI|',
            $uri->getValue(),
            $this->context
        );

        return $this;
    }
}
