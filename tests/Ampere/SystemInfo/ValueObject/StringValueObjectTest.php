<?php

declare(strict_types=1);

namespace App\Tests\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;
use App\Ampere\SystemInfo\ValueObject\StringValueObject;
use PHPUnit\Framework\TestCase;

final class StringValueObjectTest extends TestCase
{
    public function testInvalidValueThrowsValueObjectException(): void
    {
        $this->expectException(ValueObjectException::class);

        (new StringValueObject(''));
    }
}
