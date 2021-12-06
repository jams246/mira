<?php

namespace App\Ampere\DockerClient\ValueObject;

use App\Ampere\DockerClient\ValueObject\Exception\ValueObjectException;

class ContainerIdValueObject extends StringValueObject
{
    public function __construct(string $value)
    {
        if (64 != \strlen($value)) {
            throw new ValueObjectException('Container id length must be 64.');
        }
        parent::__construct($value);
    }
}
