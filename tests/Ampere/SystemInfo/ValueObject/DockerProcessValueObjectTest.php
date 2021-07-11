<?php

namespace App\Tests\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\DockerProcessValueObject;
use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;
use PHPUnit\Framework\TestCase;

class DockerProcessValueObjectTest extends TestCase
{
    public function testInvalidValueThrowsValueObjectException(): void
    {
        $this->expectException(ValueObjectException::class);

        (new DockerProcessValueObject('Test1', 'Test2', -1.0, 246));
    }

    /**
     * @dataProvider validDockerProcessValueProvider
     */
    public function testGetValues(string $name, string $state, float $cpu, int $memory, array $expected): void
    {
        $object = new DockerProcessValueObject($name, $state, $cpu, $memory);

        $this->assertSame($expected[0], $object->getName());
        $this->assertSame($expected[1], $object->getState());
        $this->assertSame($expected[2], $object->getCpu());
        $this->assertSame($expected[3], $object->getMemory());
    }

    public function validDockerProcessValueProvider(): array
    {
        return [
            [
                'name',
                'state',
                1.0,
                246,
                ['name', 'state', 1.0, 246],
            ],
        ];
    }
}
