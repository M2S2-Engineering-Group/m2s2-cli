import { Component } from '@angular/core';
import { PageHeaderComponent } from '@m2s2/ng-lib';
import { PageHeaderConfig } from '@m2s2/ng-lib';

@Component({
  selector: 'app-about',
  template: `<m2s2-page-header [config]="header" />`,
  standalone: true,
  imports: [PageHeaderComponent],
})
export class AboutComponent {
  readonly header: PageHeaderConfig = {
    title: 'About',
    subtitle: 'Learn more about this project.',
  };
}
