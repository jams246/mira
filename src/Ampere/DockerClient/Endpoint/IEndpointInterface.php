<?php

namespace App\Ampere\DockerClient\Endpoint;

use App\Ampere\DockerClient\ValueObject\MethodValueObject;
use App\Ampere\DockerClient\ValueObject\UriValueObject;

interface IEndpointInterface
{
    public function getMethod(): MethodValueObject;

    public function getUri(): UriValueObject;

    public function isStreamBlocked(): bool;
}
