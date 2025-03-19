import { Logger } from '@nestjs/common';
import { NestFactory } from '@nestjs/core';
import { NestExpressApplication } from '@nestjs/platform-express';
import { join } from 'path';
import { AppModule } from './app.module';
import { checkAndCreateTables } from './initDB';

async function bootstrap(): Promise<void> {
  const app = await NestFactory.create<NestExpressApplication>(AppModule);
  Logger.log(`listen port ${process.env.PORT}`);
  Logger.log('create tables');
  await checkAndCreateTables();
  app.useStaticAssets(join(__dirname, '../../web'));
  await app.listen(process.env.PORT);
}

bootstrap();
