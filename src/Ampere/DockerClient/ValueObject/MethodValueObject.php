<?php

namespace App\Ampere\DockerClient\ValueObject;

use App\Ampere\DockerClient\ValueObject\Exception\ValueObjectException;

class MethodValueObject extends StringValueObject
{
    private const ALLOWED_METHODS = ['GET'];

    public function __construct(string $method = 'GET')
    {
        if (!\in_array($method, self::ALLOWED_METHODS)) {
            $errorMessage = \sprintf(
                '%s only accepts allowed values. Input was %s',
                __CLASS__,
                $method
            );
            throw new ValueObjectException($errorMessage);
        }
        parent::__construct($method);
    }
}
